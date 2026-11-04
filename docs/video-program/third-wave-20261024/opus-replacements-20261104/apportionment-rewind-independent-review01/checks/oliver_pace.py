"""Whole-line Oliver pace from the earlier apportionment takes' WAV headers, as a rough reference only.

    python3 -I -B oliver_pace.py     (from the repo root)

Those takes are of the earlier script, not of 12_8. Reads headers and receipts; decodes no audio.
"""
import json
import pathlib
import statistics
import wave

A = pathlib.Path("narration_lab/third-wave-20261024/root/elevenlabs-apportionment-20261103")
audit = json.loads((A / "full-takes-audit01.json").read_text())
rates, unread = [], []
for rec in audit["records"]:
    r = json.loads(pathlib.Path(rec["receipt"]).read_text())
    words = len(r["submitted_text"].split())
    paths = []
    for key, val in r.items():
        if isinstance(val, list):
            for item in val:
                if isinstance(item, dict):
                    paths += [v for v in item.values() if isinstance(v, str) and v.endswith(".wav")]
                elif isinstance(item, str) and item.endswith(".wav"):
                    paths.append(item)
    if not paths:
        paths = [p for p in rec.values() if isinstance(p, str) and p.endswith(".wav")]
    for p in paths:
        q = pathlib.Path(p)
        if not q.is_absolute() and not q.exists():
            q = pathlib.Path(rec["receipt"]).parent / p
        try:
            with wave.open(str(q)) as w:
                rates.append((words / (w.getnframes() / w.getframerate()), rec["cue"], words))
        except Exception as e:
            unread.append((rec["cue"], str(p)[-50:], type(e).__name__))
print("takes read:", len(rates), "unread:", len(unread), unread[:2])
if rates:
    v = sorted(x[0] for x in rates)
    print(f"words per second, whole take: min {v[0]:.2f}, median {statistics.median(v):.2f}, max {v[-1]:.2f}")
    print("fastest:", [(round(a, 2), c, w) for a, c, w in sorted(rates, reverse=True)[:3]])
print("first receipt keys:", list(json.loads(pathlib.Path(audit["records"][0]["receipt"]).read_text()))[:24])
