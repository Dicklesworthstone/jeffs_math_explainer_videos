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

ROOT = Path('/data/projects/jmev-stroke-probe-20261106')
Q = ROOT/'qualification01/continuous-quality16'
TREE = ROOT/'work/continuous-quality16/workspace'
C = TREE/'explainer_corpus/complex_analysis_visual'
REPO = TREE
JOB = ROOT/'jobs/csd-continuous-rebind16'
N1 = ROOT/'jobs/csd-raster-media15/narration'
N2 = JOB/'narration'
BIN = ROOT/'work/bin/production16'
MAP = Path('explainer_corpus/complex_analysis_visual/docs/narration-direction-map-frederick-stage09.json')
NEW_MAP = MAP.with_name('narration-direction-map-frederick-quality16.json')

def read(p): return json.loads(p.read_text())
def sha(p):
    with p.open('rb') as f: return hashlib.file_digest(f, 'sha256').hexdigest()
def write(p, value):
    with p.open('x') as f: json.dump(value, f, indent=2)
def run(name, args):
    result = subprocess.run([str(x) for x in args], cwd=ROOT/'work/production-repo07', capture_output=True, timeout=900)
    for suffix, data in [('stdout', result.stdout), ('stderr', result.stderr)]:
        with (JOB/(name+'.'+suffix)).open('xb') as f: f.write(data)
    assert result.returncode == 0, (name, result.stderr.decode()[-1800:])
    return result.stdout

assert sys.argv[1:] == ['--run']
admission = read(JOB/'resource-receipt.json')
assert admission['admitted'] and 'exit' not in admission
build = ROOT/'jobs/csd-continuous-quality-build16'
receipt = read(build/'resource-receipt.json')
assert receipt['exit'] == 0 and receipt['inputs_sha256'] == receipt['final_inputs_sha256'] and not receipt.get('resource_stop')
qualification = read(build/'qualification.json')
assert sha(BIN) == qualification['binary_sha256']
# A build alone must not release the owner-rejected blocky picture candidate.
review=read(Q/'native-detail16-reviewed.json')
assert review['binary_sha256']==qualification['binary_sha256']
assert review['decision']=='pass_native_detail_for_pipeline'
assert review['actual_viewed_frames'] and review['native_dimensions']==[3840,2160]
assert not review.get('unresolved_picture_defects')

# Diagnostics and sheet extraction must have terminal stable receipts before metadata changes.
for name in ['csd-continuous-detail16-04_reading_color','csd-continuous-detail16-05_gallery','csd-continuous-sheets16']:
    r=read(ROOT/'jobs'/name/'resource-receipt.json')
    assert r['exit']==0 and r['inputs_sha256']==r['final_inputs_sha256'] and not r.get('resource_stop')
original=read(TREE/MAP)
assert original==read(Q/'before-rebind-map16.json')
data=read(Q/'narration-direction-map-frederick-quality16.json')
checks=read(Q/'marker-relocation-audit01.json')['rows']
assert len(checks)==28 and len(data['visual_holds'])==23 and len(data['helpers'])==5
for key in ['visual_holds','helpers']:
    audit=[x for x in checks if x['kind']==key]
    for row,prior,check in zip(data[key],original[key],audit):
        assert row['path']==prior['path']==check['path'] and row['marker']==prior['marker']==check['marker']
        assert prior['sha256']==check['old_sha256'] and prior['line']==check['old_line']
        f=TREE/row['path']; lines=f.read_text().splitlines(); idx=row['line']-1
        assert sha(f)==row['sha256']==check['new_sha256'] and row['line']==check['new_line']
        assert lines[idx].strip()==row['marker']
        if 'cue' in row:
            starts=[i for i,line in enumerate(lines[:idx+1]) if '"'+row['cue']+'"' in line]
            assert starts, row['cue']
            assert not any('"'+row['next_cue']+'"' in line for line in lines[max(starts):idx])
for key in original:
    if key not in ['visual_holds','helpers']: assert original[key]==data[key]
assert not (TREE/NEW_MAP).exists()
with (TREE/NEW_MAP).open('xb') as f:f.write((Q/'narration-direction-map-frederick-quality16.json').read_bytes())
canonical_path = C/'narration_units.json'
canonical_bytes = canonical_path.read_bytes()
assert canonical_bytes == (Q/'before-rebind-units16.json').read_bytes()
with (JOB/'canonical-before.json').open('xb') as f: f.write(canonical_bytes)
canonical = json.loads(canonical_bytes)
external = read(N1/'external-units.json')
assert canonical['direction_contract'] == external['direction_contract']
contract = copy.deepcopy(external['direction_contract'])
assert contract['map']['sha256'] == sha(TREE/MAP)
contract['map'] = {'path': str(NEW_MAP), 'sha256': sha(TREE/NEW_MAP)}
external['direction_contract'] = contract
canonical['direction_contract'] = contract
# Preserve the original canonical file and change only its metadata binding.
backup = canonical_path.with_name('narration_units.before-quality16.json')
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
os.chdir(ROOT/'work/production-repo07')
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
