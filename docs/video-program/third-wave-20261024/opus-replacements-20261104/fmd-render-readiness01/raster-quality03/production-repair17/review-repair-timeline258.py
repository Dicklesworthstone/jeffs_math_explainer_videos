"""Compare every chapter after the shared fade repair; no automatic visual approval."""
from pathlib import Path
import hashlib
import json
import os
import subprocess

R = Path('/data/projects/jmev-stroke-probe-20261106')
J = R / 'jobs/csd-review-repair-timeline258'
N = R / 'jobs/csd-review-repair-rebind258/narration'
OLD = R / 'jobs/csd-production-assembly19/narration'
B = R / 'work/bin/review-repair256'
BASE = R / 'jobs/csd-production-timeline21/timeline-output'


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


admission = json.loads((J / 'resource-receipt.json').read_text())
assert admission['admitted'] and 'exit' not in admission
qualification = json.loads((R / 'jobs/csd-review-repair-build256/qualification.json').read_text())
assert sha(B) == qualification['binary_sha256']
chapters = qualification['chapters']
assert len(chapters) == len(set(chapters)) == 15
assert set(p.name.removesuffix('.timeline.tsv') for p in BASE.glob('*.timeline.tsv')) == set(chapters)
os.sched_setaffinity(0, {0, 1, 2, 3})
os.environ.update(PATH='/usr/bin:/bin', TMPDIR=str(R / 'work/tmp'), XDG_CACHE_HOME=str(R / 'work/cache'))
with (J / 'native.stdout').open('xb') as out, (J / 'native.stderr').open('xb') as err:
    subprocess.run([
        str(B), 'render', *chapters, '--timeline-only', '--fps', '60', '--threads', '4',
        '--narration', str(N), '--out', str(J / 'timeline-output'),
    ], cwd=R / 'work/production-repo07', stdout=out, stderr=err, check=True, timeout=172800)
rows = []
frames = {}
for chapter in chapters:
    for kind in ('timeline', 'cues', 'cuelog'):
        old = BASE / (chapter + '.' + kind + '.tsv')
        new = J / 'timeline-output' / old.name
        assert old.read_bytes().replace(str(OLD).encode(), str(N).encode()) == new.read_bytes(), (chapter, kind)
        rows.append(dict(chapter=chapter, kind=kind, old_sha256=sha(old), new_sha256=sha(new), equal_after_named_narration_path_mapping=True))
    lines = (J / 'timeline-output' / (chapter + '.timeline.tsv')).read_text().splitlines()
    frames[chapter] = next(int(line.split('\t')[1]) for line in lines if line.startswith('frames\t'))
assert sum(frames.values()) == 133662
with (J / 'comparison.json').open('x') as stream:
    json.dump(dict(tables=rows, frames=frames, named_path_mapping={str(OLD): str(N)}, scope='All 45 complete chapter tables; repaired pictures, motion, 4K and audio review remain separate.'), stream, indent=2)
