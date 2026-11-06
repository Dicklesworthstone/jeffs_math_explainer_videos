"""Prepare a complete, disjoint two-host assignment; never launch or approve video."""
import hashlib
import json
import math
from pathlib import Path
import re

HERE = Path(__file__).resolve().parent

def validate(routing, chapters):
    expected = [r['chapter'] for r in chapters]
    assert len(expected) == len(set(expected)) == 15
    rows = routing['assignments']
    assert len(rows) == len(expected)
    assert [r['chapter'] for r in rows] == expected
    assert {r['host'] for r in rows} == {'csd', 'fmd'}
    for row, source in zip(rows, chapters):
        assert row['frames'] == source['frames']
        assert row['timeline_sha256'] == source['timeline_sha256']
        assert math.isfinite(row['proxy_seconds']) and row['proxy_seconds'] > 0
    assert routing['launch_authorized'] is False
    return True

def build(summary, log):
    chapters = summary['chapters']
    measured = {}
    for line in log.splitlines():
        match = re.match(r'^(\d\d_\w+)\s+(\d+) frames\s+([\d.]+)s\s+', line)
        if match:
            key, frames, seconds = match.groups()
            assert key not in measured, 'Ambiguous repeated chapter measurement'
            measured[key] = (int(frames), float(seconds))
    assert set(measured) == {r['chapter'] for r in chapters}
    totals = {'csd': 0.0, 'fmd': 0.0}
    hosts = {}
    for row in sorted(chapters, key=lambda r: (-measured[r['chapter']][1], r['chapter'])):
        chapter = row['chapter']
        frames, seconds = measured[chapter]
        assert frames == row['frames'] and math.isfinite(seconds) and seconds > 0
        host = min(totals, key=lambda h: (totals[h], h))
        hosts[chapter] = host
        totals[host] += seconds
    routing = dict(assignments=[dict(chapter=row['chapter'], host=hosts[row['chapter']],
        frames=row['frames'], timeline_sha256=row['timeline_sha256'],
        proxy_seconds=measured[row['chapter']][1]) for row in chapters],
        proxy_totals=totals, launch_authorized=False,
        scope='LPT balance using completed low-resolution timeline elapsed times only. '
              'Not a 4K runtime estimate or host-speed benchmark. Actual picture review, '
              'guarded per-host launch, output import and distributed master integration remain required.')
    validate(routing, chapters)
    return routing

if __name__ == '__main__':
    summary_path = HERE/'csd-timeline-summary01.json'
    log_path = HERE/'csd-timeline-native01.stdout'
    result = build(json.loads(summary_path.read_text()), log_path.read_text())
    result['inputs'] = {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                        for p in (summary_path, log_path, Path(__file__))}
    with (HERE/'chapter-assignment01.json').open('x') as out:
        json.dump(result, out, indent=2)
    print(json.dumps(result, indent=2))
