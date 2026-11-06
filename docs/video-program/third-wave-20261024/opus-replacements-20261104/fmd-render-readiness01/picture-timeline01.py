"""Render repaired native schedules and compare every row before score reuse."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path('/data/projects/jmev-stroke-probe-20261106')
TREE = ROOT/'work/picture-repair03/workspace'
C = TREE/'explainer_corpus/complex_analysis_visual'
BIN = ROOT/'work/bin/production08c'
N = ROOT/'jobs/stage08-picture-rebind01/narration'
OLD_N = ROOT/'jobs/stage07-narration-derive01/narration'
BASE = ROOT/'jobs/stage07-timeline01/timeline-output'
JOB = ROOT/'jobs/stage08-picture-timeline01'
OUT = JOB/'timeline-output'

def read(p): return json.loads(p.read_text())
def sha(p):
    with p.open('rb') as f: return hashlib.file_digest(f, 'sha256').hexdigest()
def success(name):
    r = read(ROOT/'jobs'/name/'resource-receipt.json')
    assert r['exit'] == 0 and r['inputs_sha256'] == r['final_inputs_sha256'] and not r.get('resource_stop')
def run(name, args):
    with (JOB/(name+'.stdout')).open('xb') as out, (JOB/(name+'.stderr')).open('xb') as err:
        p = subprocess.run([str(x) for x in args], cwd=ROOT/'work/production-repo07', stdout=out, stderr=err, timeout=172800)
    assert p.returncode == 0, (name, p.returncode)
    return (JOB/(name+'.stdout')).read_bytes()

assert sys.argv[1:] == ['--run']
admission = read(JOB/'resource-receipt.json')
assert admission['admitted'] and 'exit' not in admission
for name in ['stage08-picture-repair-build03', 'stage08-picture-rebind01', 'stage07-timeline01']:
    success(name)
qualification = read(ROOT/'jobs/stage08-picture-repair-build03/qualification.json')
assert sha(BIN) == qualification['binary_sha256']
chapters = run('registry', [BIN, 'chapters']).decode().splitlines()
assert chapters == qualification['chapters'] and len(chapters) == 15
run('coherent', [sys.executable, '-I', '-B', TREE/'tools/coherent_narration.py', 'verify', N,
                '--script', C/'script.tsv', '--units', N/'external-units.json', '--require-directions'])
assert not OUT.exists()
os.environ.update(TMPDIR=str(ROOT/'work/tmp'), XDG_CACHE_HOME=str(ROOT/'work/cache'))
run('native', [BIN, 'render', 'all', '--timeline-only', '--fps', '60', '--narration', N, '--out', OUT])
assert {p.name for p in OUT.glob('*.timeline.tsv')} == {c+'.timeline.tsv' for c in chapters}
comparisons = []
for chapter in chapters:
    for kind in ['timeline', 'cues', 'cuelog']:
        old, new = BASE/(chapter+'.'+kind+'.tsv'), OUT/(chapter+'.'+kind+'.tsv')
        expected = old.read_bytes().replace(str(OLD_N).encode(), str(N).encode())
        assert expected == new.read_bytes(), ('Full table differs', chapter, kind)
        comparisons.append({'chapter': chapter, 'table': kind, 'old_sha256': sha(old), 'new_sha256': sha(new),
                            'full_rows_equal_after_exact_narration_directory_substitution': True})
with (JOB/'comparison.json').open('x') as f:
    json.dump({'tables': comparisons, 'table_count': 45, 'score_reuse': 'Original score terminal receipt and all 15 QA must still be checked.',
               'scope': 'Complete native schedules only. Repaired pictures, native4K detail, listening, fine sync and independent AV are not accepted.'}, f, indent=2)
print('All 45 complete schedule tables match after exact audio-directory substitution. Picture review remains required.')
