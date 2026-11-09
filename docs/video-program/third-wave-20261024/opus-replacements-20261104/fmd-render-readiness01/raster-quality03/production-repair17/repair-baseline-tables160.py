import base64
import hashlib
import json
from pathlib import Path

R = Path('/data/projects/jmev-hz2-render-worker-20261107')
Q = R/'qualification01/production19'
J = R/'jobs/hz2-repair-baseline-tables160'
D = Path('/data/projects/jmev-stroke-probe-20261106')
r = json.loads((J/'resource-receipt.json').read_text())
assert r['admitted'] and 'exit' not in r
assert D.resolve() == R/'work/frozen-runtime47/data/projects/jmev-stroke-probe-20261106'
rows = json.loads((Q/'repair-baseline-tables160.json').read_text())['files']
assert len(rows) == len({x['path'] for x in rows}) == 6
for x in rows:
    p = Path(x['path'])
    assert p.parent == D/'jobs/csd-production-timeline21/timeline-output'
    assert p.name in {c+'.'+s for c in ('08_loop_sums', '09_boundary') for s in ('timeline.tsv', 'cues.tsv', 'cuelog.tsv')}
    data = base64.b64decode(x['base64'], validate=True)
    assert hashlib.sha256(data).hexdigest() == x['sha256']
    if p.exists():
        assert p.read_bytes() == data
    else:
        with p.open('xb') as f:
            f.write(data)
    assert hashlib.sha256(p.read_bytes()).hexdigest() == x['sha256']
print('All six original tables exact; existing files preserved')
