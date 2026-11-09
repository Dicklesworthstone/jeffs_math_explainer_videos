"""Targeted 09 schedule comparison for build313; no automatic visual approval.

Adapted from review-repair-timeline269.py by cc0 (not run by cc0). Only boundary.rs changed, so
only 09_boundary is rendered timeline-only and its three tables are compared byte for byte with
timeline269's (under the named rebind269 -> rebind313 narration path mapping) and with
production timeline21's (assembly19 -> rebind313). The other fourteen chapters are not re-run
under binary 313; see cc0-repair313-notes.md.
"""
from pathlib import Path
import hashlib
import json
import os
import subprocess

R = Path('/data/projects/jmev-stroke-probe-20261106')
J = R / 'jobs/csd-review-repair-timeline313'
N = R / 'jobs/csd-review-repair-rebind313/narration'
B = R / 'work/bin/review-repair313'
T269 = R / 'jobs/csd-review-repair-timeline269'
N269 = R / 'jobs/csd-review-repair-rebind269/narration'
BASE21 = R / 'jobs/csd-production-timeline21/timeline-output'
OLD21 = R / 'jobs/csd-production-assembly19/narration'
CHAPTERS = ['09_boundary']
FRAMES = {'09_boundary': 16504}


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


admission = json.loads((J / 'resource-receipt.json').read_text())
assert admission['admitted'] and 'exit' not in admission
qualification = json.loads((R / 'jobs/csd-review-repair-build313/qualification.json').read_text())
assert sha(B) == qualification['binary_sha256']
assert set(CHAPTERS) <= set(qualification['chapters'])
c269 = json.loads((T269 / 'comparison.json').read_text())
assert c269['named_path_mapping'] == {str(OLD21): str(N269)}
rows269 = {(r['chapter'], r['kind']): r for r in c269['tables']}
os.sched_setaffinity(0, {0, 1, 2, 3})
os.environ.update(PATH='/usr/bin:/bin', TMPDIR=str(R / 'work/tmp'), XDG_CACHE_HOME=str(R / 'work/cache'))
with (J / 'native.stdout').open('xb') as out, (J / 'native.stderr').open('xb') as err:
    subprocess.run([
        str(B), 'render', *CHAPTERS, '--timeline-only', '--fps', '60', '--threads', '4',
        '--narration', str(N), '--out', str(J / 'timeline-output'),
    ], cwd=R / 'work/production-repo07', stdout=out, stderr=err, check=True, timeout=172800)
# timeline269 never listed its output folder, so other files are recorded, not refused
outputs = sorted(p.name for p in (J / 'timeline-output').iterdir())
rows = []
frames = {}
for chapter in CHAPTERS:
    for kind in ('timeline', 'cues', 'cuelog'):
        name = chapter + '.' + kind + '.tsv'
        new = J / 'timeline-output' / name
        old = T269 / 'timeline-output' / name
        row269 = rows269[(chapter, kind)]
        assert sha(old) == row269['new_sha256'] and row269['equal_after_named_narration_path_mapping']
        assert old.read_bytes().replace(str(N269).encode(), str(N).encode()) == new.read_bytes(), (chapter, kind, 'timeline269')
        assert (BASE21 / name).read_bytes().replace(str(OLD21).encode(), str(N).encode()) == new.read_bytes(), (chapter, kind, 'timeline21')
        rows.append(dict(chapter=chapter, kind=kind, timeline269_sha256=sha(old), production21_sha256=sha(BASE21 / name),
                         new_sha256=sha(new), equal_to_timeline269_after_named_narration_path_mapping=True,
                         equal_to_production21_after_named_narration_path_mapping=True))
    lines = (J / 'timeline-output' / (chapter + '.timeline.tsv')).read_text().splitlines()
    frames[chapter] = next(int(line.split('\t')[1]) for line in lines if line.startswith('frames\t'))
    assert frames[chapter] == c269['frames'][chapter] == FRAMES[chapter]
with (J / 'comparison.json').open('x') as stream:
    json.dump(dict(tables=rows, frames=frames, outputs=outputs, named_path_mapping={str(N269): str(N)},
                   production_path_mapping={str(OLD21): str(N)},
                   scope='Chapter 09 tables only, under binary 313; repaired pictures, motion, 4K and audio '
                         'review remain separate. The other fourteen chapters were not re-run.'), stream, indent=2)
print('09 timeline, cues and cuelog equal timeline269 and production timeline21 after named path mapping.')
