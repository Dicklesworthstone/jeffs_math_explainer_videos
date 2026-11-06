"""Rebind picture-only source evidence, retaining every audio sample and timing."""
import copy
import difflib
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
OLD = ROOT/'work/picture-repair03/workspace'
TREE = ROOT/'work/raster-quality-fmd04/workspace'
C = TREE/'explainer_corpus/complex_analysis_visual'
REPO = TREE
JOB = ROOT/'jobs/fmd-raster-rebind11'
N1 = ROOT/'jobs/fmd-picture-narration01/narration'
N2 = JOB/'narration'
BIN = ROOT/'work/bin/production09i'
MAP = Path('explainer_corpus/complex_analysis_visual/docs/narration-direction-map-frederick-stage08.json')
NEW_MAP = MAP.with_name('narration-direction-map-frederick-stage09.json')

def read(p): return json.loads(p.read_text())
def sha(p):
    with p.open('rb') as f: return hashlib.file_digest(f, 'sha256').hexdigest()
def write(p, value):
    with p.open('x') as f: json.dump(value, f, indent=2)
def run(name, args):
    result = subprocess.run([str(x) for x in args], cwd=REPO, capture_output=True, timeout=900)
    for suffix, data in [('stdout', result.stdout), ('stderr', result.stderr)]:
        with (JOB/(name+'.'+suffix)).open('xb') as f: f.write(data)
    assert result.returncode == 0, (name, result.stderr.decode()[-1800:])
    return result.stdout

assert sys.argv[1:] == ['--run']
admission = read(JOB/'resource-receipt.json')
assert admission['admitted'] and 'exit' not in admission
build = ROOT/'jobs/fmd-raster-quality-build05'
receipt = read(build/'resource-receipt.json')
assert receipt['exit'] == 0 and receipt['inputs_sha256'] == receipt['final_inputs_sha256'] and not receipt.get('resource_stop')
qualification = read(build/'qualification.json')
assert sha(BIN) == qualification['binary_sha256']
# A build alone must not release the owner-rejected blocky picture candidate.
review=read(ROOT/'qualification01/raster-quality03/native-detail11-reviewed.json')
assert review['binary_sha256']==qualification['binary_sha256']
assert review['decision']=='pass_native_detail_for_pipeline'
assert review['actual_viewed_frames'] and review['native_dimensions']==[3840,2160]
assert not review.get('unresolved_picture_defects')

data = read(OLD/MAP)
assert read(TREE/MAP) == data
original = copy.deepcopy(data)
checks = []
for row in data['visual_holds'] + data['helpers']:
    old_path, new_path = OLD/row['path'], TREE/row['path']
    assert sha(old_path) == row['sha256']
    old_lines, new_lines = old_path.read_text().splitlines(), new_path.read_text().splitlines()
    old_index = row['line'] - 1
    assert old_lines[old_index].strip() == row['marker']
    blocks = difflib.SequenceMatcher(None, old_lines, new_lines, autojunk=False).get_matching_blocks()
    matches = [b.b + old_index - b.a for b in blocks if b.a <= old_index < b.a+b.size]
    assert len(matches) == 1, ('changed hold marker', row)
    new_index = matches[0]
    assert new_lines[new_index].strip() == row['marker']
    # The mapped call must remain inside the same explicit cue interval.
    if 'cue' in row:
        cue_token = '"'+row['cue']+'"'
        next_token = '"'+row['next_cue']+'"'
        old_starts = [i for i,s in enumerate(old_lines) if cue_token in s and i <= old_index]
        new_starts = [i for i,s in enumerate(new_lines) if cue_token in s and i <= new_index]
        assert old_starts and new_starts, ('cue start', row['cue'])
        assert not any(next_token in s for s in old_lines[max(old_starts):old_index])
        assert not any(next_token in s for s in new_lines[max(new_starts):new_index])
    checks.append({'path': row['path'], 'old_line': row['line'], 'new_line': new_index+1,
                   'marker': row['marker'], 'old_sha256': row['sha256'], 'new_sha256': sha(new_path)})
    row['line'], row['sha256'] = new_index+1, sha(new_path)
assert len(data['visual_holds']) == 23 and len(data['helpers']) == 5
assert data['overrides'] == original['overrides']
data['scope'] = 'Raster quality stage09: existing 23 hold calls and five helper markers checked against source. No narration, listening or timing acceptance.'
write(TREE/NEW_MAP, data)
canonical_path = C/'narration_units.json'
canonical_bytes = canonical_path.read_bytes()
assert canonical_bytes == (OLD/'explainer_corpus/complex_analysis_visual/narration_units.json').read_bytes()
with (JOB/'canonical-before.json').open('xb') as f: f.write(canonical_bytes)
canonical = json.loads(canonical_bytes)
external = read(N1/'external-units.json')
assert canonical['direction_contract'] == external['direction_contract']
contract = copy.deepcopy(external['direction_contract'])
assert contract['map']['sha256'] == sha(OLD/MAP)
contract['map'] = {'path': str(NEW_MAP), 'sha256': sha(TREE/NEW_MAP)}
external['direction_contract'] = contract
canonical['direction_contract'] = contract
# Preserve the original canonical file and change only its metadata binding.
backup = canonical_path.with_name('narration_units.before-raster-stage09.json')
with backup.open('xb') as f: f.write(canonical_bytes)
canonical_path.write_text(json.dumps(canonical, indent=2))
N2.mkdir()
for p in N1.iterdir(): shutil.copy2(p, N2/p.name)
(N2/'external-units.json').write_text(json.dumps(external, indent=2))
assembly, report = read(N1/'assembly.json'), read(N1/'report.json')
assert assembly['report_sha256'] == sha(N1/'report.json')
old_report, old_assembly = copy.deepcopy(report), copy.deepcopy(assembly)
plan_hash = sha(N2/'external-units.json')
report['meta']['units_sha256'] = assembly['manifest']['units_sha256'] = plan_hash
holds = {h['cue']: h for h in data['visual_holds']}
for disposition in assembly['manifest']['direction_dispositions']:
    h = holds[disposition['cue']]
    assert disposition['kind'] == 'native_visual_hold'
    disposition['evidence'] = {'path': str(TREE/h['path']), 'sha256': h['sha256']}
    disposition['reason'] = 'Existing native visual hold retained at '+h['path']+':'+str(h['line'])+'. No samples or silence changed; listening unreviewed.'
os.chdir(REPO)
spec = importlib.util.spec_from_file_location('coherent', TREE/'tools/coherent_narration.py')
coherent = importlib.util.module_from_spec(spec)
spec.loader.exec_module(coherent)
script, units = coherent.read_units(C/'script.tsv', N2/'external-units.json', True)
binding = coherent.direction_contract(C/'script.tsv', external)
assert binding['resolved'] == old_report['meta']['direction_contract']['binding']['resolved']
report['meta']['direction_contract'] = coherent.review_directions(binding, units, assembly['manifest'])
assert report['lines'] == old_report['lines']
assert assembly['outputs'] == old_assembly['outputs'] and assembly['manifest']['units'] == old_assembly['manifest']['units']
(N2/'report.json').write_text(json.dumps(report, indent=2))
assembly['report_sha256'] = sha(N2/'report.json')
(N2/'assembly.json').write_text(json.dumps(assembly, indent=2))
changed = {p.name for p in N1.iterdir() if sha(p) != sha(N2/p.name)}
assert changed == {'external-units.json', 'report.json', 'assembly.json'}
assert len(list(N2.glob('*.wav'))) == 105
run('coherent', [sys.executable, '-I', '-B', TREE/'tools/coherent_narration.py', 'verify', N2,
                '--script', C/'script.tsv', '--units', N2/'external-units.json', '--require-directions'])
args = [sys.executable, '-I', '-B', TREE/'tools/narration_delivery.py', '--topic', C,
        '--narration', N2, '--consumer', 'render', '--gain-db', '0']
for chapter in qualification['chapters']: args += ['--chapter-key', chapter]
delivery = json.loads(run('delivery', args))
assert delivery['verified_cues'] == delivery['selected_cues'] == 105 and delivery['plan_mode'] == 'external-ui'
write(JOB/'rebind-summary.json', {'holds': 23, 'helpers': 5, 'marker_checks': checks,
    'wav_files_byte_identical': 105, 'unchanged_files': len(list(N1.iterdir()))-len(changed),
    'report_sentence_rows_unchanged': True, 'generation_records_unchanged': True,
    'delivery': delivery, 'map_sha256': sha(TREE/NEW_MAP), 'canonical_sha256': sha(canonical_path),
    'scope': 'Source bindings and sample reconstruction only; full timeline equality, repaired pictures, native4K detail, listening, fine sync and independent AV remain open.'})
print('Rebound 23 holds and five helper markers; all 105 complete WAVs and sentence/phrase timing retained.')
