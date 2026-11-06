"""Package exact completed FMD review inputs for the CSD chapter worker.

Run inside a separately admitted resource guard after actual picture review.
This neither manufactures a review nor launches a renderer.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import sys

HERE = Path('/data/projects/jmev-fmd-render-qualification-20261106-1111/qualification01/raster-quality03')
ROOT = HERE.parent.parent
JOB = ROOT/'jobs/fmd-raster-transfer12'

def load(name):
    spec = importlib.util.spec_from_file_location(name, HERE/name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

def read(path):
    return json.loads(path.read_text())

def sha(path):
    with path.open('rb') as handle:
        return hashlib.file_digest(handle, 'sha256').hexdigest()

def run():
    assert Path(__file__).resolve().parent == HERE
    gate = load('final-review-gate12.py')
    P, S = gate.P, gate.S
    admission = read(JOB/'resource-receipt.json')
    assert admission['admitted'] and 'exit' not in admission
    assert admission['policy'] == S.POLICY
    review = HERE/'preview-reviewed12.json'
    verdict = gate.check(review)
    permit = read(HERE/'distributed-review-permit12.json')
    assignment = HERE/'chapter-assignment12.json'
    assert permit['review_gate_result'] == verdict
    assert permit['assignment_sha256'] == sha(assignment)
    assert permit['binary_sha256'] == P.BINARY_SHA
    assert permit['chapters'] == list(S.CHAPTERS)
    sources = {
        'preview-summary.json': P.JOB/'preview-summary.json',
        'extraction.json': gate.X.PICTURES_OUT/'extraction.json',
        'comparison.json': ROOT/'jobs/fmd-raster-timeline12/comparison.json',
        'distributed-review-permit12.json': HERE/'distributed-review-permit12.json',
        'chapter-assignment12.json': assignment,
    }
    for phase in ('preview', 'pictures', 'timeline'):
        S.success('fmd-raster-'+phase+'12')
        sources[phase+'-resource-receipt.json'] = ROOT/'jobs'/('fmd-raster-'+phase+'12')/'resource-receipt.json'
    comparison = read(sources['comparison.json'])
    assert comparison['table_count'] == len(comparison['tables']) == 45
    assert {(r['chapter'], r['table']) for r in comparison['tables']} == {
        (c, kind) for c in S.CHAPTERS for kind in ('timeline', 'cues', 'cuelog')}
    for row in comparison['tables']:
        assert row['full_rows_equal_after_exact_narration_directory_substitution'] is True
        rel = 'timeline-output/'+row['chapter']+'.'+row['table']+'.tsv'
        source = ROOT/'jobs/fmd-raster-timeline12'/rel
        assert sha(source) == row['new_sha256']
        sources[rel] = source
    frozen = {rel: sha(source) for rel, source in sources.items()}
    out = JOB/'production-inputs-raster12'
    out.mkdir(exist_ok=False)
    for rel, source in sources.items():
        target = out/rel
        target.parent.mkdir(parents=True, exist_ok=True)
        with source.open('rb') as src, target.open('xb') as dst:
            shutil.copyfileobj(src, dst)
        assert sha(target) == frozen[rel] == sha(source)
    assert gate.check(review) == verdict
    manifest = dict(origin_host='fmd', binary_sha256=P.BINARY_SHA,
        files=[dict(relative_path=rel, sha256=frozen[rel]) for rel in sorted(sources)],
        scope='Exact transfer only; actual review provenance retained, no CSD listening or final acceptance.')
    with (out/'transfer.json').open('x') as handle:
        json.dump(manifest, handle, indent=2)
    print('Prepared exact reviewed transfer:', len(sources), 'files; no renderer launched.')

if __name__ == '__main__':
    assert sys.argv[1:] == ['--run']
    run()
