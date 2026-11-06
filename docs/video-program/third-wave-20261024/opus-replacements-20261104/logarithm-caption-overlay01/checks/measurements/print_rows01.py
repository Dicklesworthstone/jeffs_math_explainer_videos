"""Print where each measured row's ink changes, frame by frame, for each short run."""
import json
import sys

runs = json.load(open(sys.argv[1]))
FPS = 60
for run in runs:
    # The first decoded frame is the first with pts >= start.
    base = round(run['start'] * FPS + 0.4999)
    print(f"--- run {run['start']} +{run['span']} frames {run['frames']} first frame {base} = {base / FPS:.3f}")
    for label, series in run['rows'].items():
        low, high = min(series), max(series)
        spread = high - low
        if spread < 0.003 * high:
            print(f'  {label}: flat {low}')
            continue
        moved = [i for i in range(1, len(series)) if abs(series[i] - series[i - 1]) > 0.02 * spread]
        first, last = moved[0], moved[-1]
        print(f'  {label}: low {low} high {high} moves {base + first}..{base + last} '
              f'= {(base + first) / FPS:.3f}..{(base + last) / FPS:.3f} ({len(moved)} steps) '
              f'head {series[0]} tail {series[-1]}')
