"""Reassemble retained PCM on FMD with local absolute source paths, unchanged timing."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
from types import SimpleNamespace

ROOT = Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
WORKSPACE = ROOT/'work/production-build07/workspace'
INPUTS = ROOT/'production-inputs01'
OLD = INPUTS/'narration'
JOB = ROOT/'jobs/fmd-narration-rebind01'
OUT = JOB/'narration'

def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

receipt = json.loads((JOB/'resource-receipt.json').read_text())
assert receipt['admitted'] and 'exit' not in receipt
assert receipt['policy'] == dict(admission_gib=10, stop_free_gib=8,
    phase_growth_gib=4, work_cap_gib=56, target_growth_cap_gib=20,
    sample_seconds=10)
os.chdir(INPUTS/'production-repo07')
assembly = json.loads((OLD/'assembly.json').read_text())
spans = JOB/'original-spans.json'
with spans.open('x') as f:
    json.dump(assembly['manifest'], f, indent=2)
spec = importlib.util.spec_from_file_location('coherent', WORKSPACE/'tools/coherent_narration.py')
coherent = importlib.util.module_from_spec(spec)
spec.loader.exec_module(coherent)
script = WORKSPACE/'explainer_corpus/complex_analysis_visual/script.tsv'
coherent.assemble(SimpleNamespace(script=script, units=OLD/'external-units.json',
    require_directions=True, chapter=None, spans=spans, out=OUT))
old_report = json.loads((OLD/'report.json').read_text())
new_report = json.loads((OUT/'report.json').read_text())
assert old_report['lines'].keys() == new_report['lines'].keys()
assert len(new_report['lines']) == 105
for cue, rows in old_report['lines'].items():
    updated = new_report['lines'][cue]
    assert len(rows) == len(updated)
    for a, b in zip(rows, updated):
        assert {k:v for k,v in a.items() if k != 'source'} == {k:v for k,v in b.items() if k != 'source'}, cue
        assert sha(Path(b['source'])) == a['source_sha256'], cue
    assert sha(OLD/(cue+'.wav')) == sha(OUT/(cue+'.wav')), cue
for name in ('external-units.json', 'phrase-timings.json'):
    with (OUT/name).open('xb') as f:
        f.write((OLD/name).read_bytes())
    assert sha(OUT/name) == sha(OLD/name)
chapters = subprocess.check_output([str(ROOT/'production07d'), 'chapters'], text=True).splitlines()
assert len(chapters) == 15 and len(set(chapters)) == 15
command = ['/usr/bin/python3', '-I', '-B', str(WORKSPACE/'tools/narration_delivery.py'),
    '--topic', str(script.parent), '--narration', str(OUT), '--consumer', 'render',
    '--gain-db', '0', '--score', str(INPUTS/'scored/stems')]
for chapter in chapters:
    command.extend(['--chapter-key', chapter])
result = subprocess.run(command, check=True, text=True, capture_output=True, timeout=600)
verified = json.loads(result.stdout)
assert verified['verified_cues'] == verified['selected_cues'] == 105
with (JOB/'rebind-summary.json').open('x') as f:
    json.dump({'unchanged_full_wavs':105, 'timings_identical':True,
        'sentence_rows_equal_except_source_path':True, 'native_delivery':verified,
        'scope':'Local path rebinding and exact PCM reconstruction only; no listening, fine-sync or picture acceptance.'},f,indent=2)
print(result.stdout)
