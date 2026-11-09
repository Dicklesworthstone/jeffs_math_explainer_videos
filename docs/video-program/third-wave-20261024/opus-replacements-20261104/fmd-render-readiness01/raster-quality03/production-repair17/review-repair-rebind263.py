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
Q = ROOT/'qualification01/production-repair17'
TREE = ROOT/'work/review-repair262/workspace'
C = TREE/'explainer_corpus/complex_analysis_visual'
REPO = TREE
JOB = ROOT/'jobs/csd-review-repair-rebind263'
N1 = ROOT/'jobs/csd-production-assembly19/narration'
N2 = JOB/'narration'
BIN = ROOT/'work/bin/review-repair262'
MAP = Path('explainer_corpus/complex_analysis_visual/docs/narration-direction-map-frederick-production19.json')
NEW_MAP = MAP.with_name('narration-direction-map-frederick-review263.json')

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
build = ROOT/'jobs/csd-review-repair-build262'
receipt = read(build/'resource-receipt.json')
assert receipt['exit'] == 0 and receipt['inputs_sha256'] == receipt['final_inputs_sha256'] and not receipt.get('resource_stop')
qualification = read(build/'qualification.json')
assert sha(BIN) == qualification['binary_sha256']
# A build is not visual acceptance. Rebind only to make the repaired
# candidate reviewable; no render permit or publication follows automatically.
files=read(build/'source-copy.json')['files']
allowed = ('boundary.rs', 'loop_sums.rs', 'one_derivative.rs', 'mod.rs', 'rigidity.rs', 'gallery.rs', 'draw.rs', 'plate_fade_probe.rs')
assert read(build/'source-copy.json')['changed'] == sorted('explainer_corpus/complex_analysis_visual/src/chapters/' + name for name in allowed)
assert qualification['explicit_native_plate_fade_probe_passed'] is True
for name,digest in files.items(): assert sha(TREE/name)==digest,name
old=read(ROOT/'jobs/csd-production-assembly19/resource-receipt.json')
assert old.get('finished') and old.get('exit')==0 and old['inputs_sha256']==old['final_inputs_sha256'] and not old.get('resource_stop')
spec=importlib.util.spec_from_file_location('assembly19',Q/'prepare-assembly19.py')
helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
original=read(TREE/MAP)
texts=helper.script_rows(C/'script.tsv')
data,checks=helper.relocate_map(original,TREE,texts,sha(C/'script.tsv'))
# Direction, text, exact hold marker and duration stay unchanged; source lines relocate uniquely within the same cue interval.
for key in original:
 if key not in ('visual_holds','helpers','scope'): assert data[key]==original[key],key
for key in ('visual_holds','helpers'):
 for before,after in zip(original[key],data[key]):
  assert {k:v for k,v in before.items() if k not in ('sha256','line')}=={k:v for k,v in after.items() if k not in ('sha256','line')}
  if before['sha256']!=after['sha256']: assert after['path'].endswith(tuple('/chapters/' + name for name in allowed))
data['scope']='Picture-repair source rebind; no audio, timing, listening or picture acceptance.'
write(TREE/NEW_MAP,data)
canonical_path=C/'narration_units.json'
canonical_bytes=canonical_path.read_bytes()
assert sha(canonical_path)==files[str(canonical_path.relative_to(TREE))]
with (JOB/'canonical-before.json').open('xb') as f:f.write(canonical_bytes)
canonical = json.loads(canonical_bytes)
external = read(N1/'external-units.json')
assert canonical['direction_contract'] == external['direction_contract']
contract = copy.deepcopy(external['direction_contract'])
assert contract['map']['sha256'] == sha(TREE/MAP)
contract['map'] = {'path': str(NEW_MAP), 'sha256': sha(TREE/NEW_MAP)}
external['direction_contract'] = contract
canonical['direction_contract'] = contract
# Preserve the original canonical file and change only its metadata binding.
backup = canonical_path.with_name('narration_units.before-review263.json')
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
