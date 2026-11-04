"""Speaking rate of ElevenLabs Oliver on other films' assembled narration: context for the reading window only.

python3 -I -B oliver_rate.py NARRATION_ROOT

NARRATION_ROOT is a folder of runs holding */narration/report.json. A report counts only if its text names ElevenLabs
or Oliver. Each spoken sentence's rate comes from its own source sample span and the report's sample rate, so inserted
pauses are excluded. This measures other sentences by the same voice. It does not measure 11_8 of this film.
"""
import json
import statistics
import sys
from pathlib import Path


def main():
    root = Path(sys.argv[1])
    spans = {}
    reports = 0
    for report in sorted(root.glob("*/narration/report.json")):
        raw = report.read_text(encoding="utf-8")
        if "elevenlabs" not in raw.lower() and "oliver" not in raw.lower():
            continue
        data = json.loads(raw)
        rate = data.get("meta", {}).get("rate") or 24000
        reports += 1
        for items in data.get("lines", {}).values():
            if not isinstance(items, list):
                continue
            for span in items:
                words = len(str(span.get("sentence", "")).split())
                if not words or "source_out_sample" not in span:
                    continue
                seconds = (span["source_out_sample"] - span["source_in_sample"]) / rate
                if seconds > 0:
                    key = (span.get("source_sha256"), span["source_in_sample"], span["source_out_sample"])
                    spans[key] = (words, seconds)
    print(f"reports naming ElevenLabs or Oliver: {reports}; distinct source spans: {len(spans)}")
    for low, high in [(1, 200), (14, 22), (16, 20)]:
        rates = sorted(w / s for w, s in spans.values() if low <= w <= high)
        if rates:
            print(f"{low}-{high} words: {len(rates)} spans; rate median {statistics.median(rates):.2f}, "
                  f"5th percentile {rates[len(rates) // 20]:.2f}, 95th {rates[len(rates) * 19 // 20]:.2f}, max {rates[-1]:.2f} words/s")
    print("The closing sentence of 11_8 has", len("Only the singular ones, which squash the plane flat, have none, for the same reason zero had none.".split()), "words.")
    for r in (2.4, 2.6, 2.8, 2.96, 3.0, 3.32, 3.4):
        print(f"  at {r:.2f} words/s: speech {18 / r:4.2f} s; clean window = speech + 0.60 tail - 0.50 fade = {18 / r + 0.1:4.2f} s; "
              f"baseline needs 5.40 s, patched needs 6.20 s")


if __name__ == "__main__":
    main()
