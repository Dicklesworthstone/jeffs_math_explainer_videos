"""Guarded source-only rebind of retained557 narration to repaired native761."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

R = Path.home() / 'scratch/jmev-complex-native-20261109'
Q = R / 'qualification'
SOURCE = R / 'source760'
W = SOURCE / 'workspace'
TOPIC = W / 'explainer_corpus/complex_analysis_visual'
OLD = R / 'jobs/mini-boundary-delivery557/narration'
JOB = R / 'jobs/mini-boundary-delivery764'
MAPREL = 'explainer_corpus/complex_analysis_visual/docs/narration-direction-map-frederick-review764.json'


def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def read(p):
    return json.loads(Path(p).read_text())


def write(p, data):
    with Path(p).open('x') as f:
        json.dump(data, f, indent=2)
        f.write('\n')


def success(p):
    d = read(p)
    assert d.get('admitted') and d.get('finished') and d.get('exit') == 0, p
    assert d['inputs_sha256'] == d['final_inputs_sha256'], p
    assert not any(d.get(k) for k in ('resource_stop', 'error', 'stopped_for_resource_limit')), p


def snapshot(root):
    rows = {}
    for p in root.rglob('*'):
        assert not p.is_symlink(), p
        if p.is_file():
            rows[p.relative_to(root).as_posix()] = sha(p)
    return rows


def main():
    helper = Q / 'root-mini-delivery540.py'
    assert sha(helper) == '23be4c05ef843e2cd40d07eb1de09205c2f5054e3370f307c2af577db021cabb'
    spec = importlib.util.spec_from_file_location('delivery540', helper)
    h = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(h)
    h.guard_binding(JOB)
    success(R / 'jobs/native-boundary761/resource-receipt.json')
    success(OLD.parent / 'resource-receipt.json')
    assert sha(R / 'jobs/native-boundary761/qualification.json') == 'b225700d230f5880663e786d83b3dc9a43d0776dc9c07fa3d669de6ab6198ba8'
    assert sha(R / 'jobs/native-boundary761/complex_analysis_visual-761') == '71736c26d9953f08af8f4ce4f6fc0f13aeb98e7135b3dd415fb3ebf00e161c26'
    before = snapshot(SOURCE)
    assert before == read(Q / 'root-boundary760-source-manifest.json')['files']
    original = snapshot(OLD)
    assert len(list(OLD.glob('*.wav'))) == 105
    assert sha(OLD / 'external-units.json') == '9308c2b230a7cafd933dc826511bc8111748e9355ab9e445d3f7f89c81511c39'
    assert sha(OLD / 'report.json') == 'f0bcf21d3051070f7ddbf5716d1f42f75faa162dbc41c39091a6fd586d805d04'
    assert sha(OLD / 'assembly.json') == 'f615a0107a65e9bb54d95451547deff055fba4289128f8895146141e3a30e4c4'
    mapfile = Q / 'root-boundary764-map.json'
    assert sha(mapfile) == 'ff852ebcff626a1dcef5bee91613eba324bba317cacbc7b0f2dd129897bc1681'
    mapping = read(mapfile)
    for row in mapping['visual_holds'] + mapping['helpers']:
        p = W / row['path']
        assert sha(p) == row['sha256']
        assert p.read_text().splitlines()[row['line']-1].strip() == row['marker'].strip()
    plan = read(OLD / 'external-units.json')
    original_plan = copy.deepcopy(plan)
    plan['direction_contract']['map'] = dict(path=MAPREL, sha256=sha(mapfile))
    canonical = TOPIC / 'narration_units.json'
    assert {k:v for k,v in read(canonical).items() if k != 'direction_contract'} == {k:v for k,v in plan.items() if k != 'direction_contract'}
    backup = TOPIC / 'narration_units.before-mini764.json'
    with backup.open('xb') as f:
        f.write(canonical.read_bytes())
    with (W / MAPREL).open('xb') as f:
        f.write(mapfile.read_bytes())
    narration = JOB / 'narration'
    narration.mkdir()
    for p in OLD.iterdir():
        assert p.is_file() and not p.is_symlink()
        if p.name not in ('report.json', 'assembly.json', 'external-units.json'):
            with (narration / p.name).open('xb') as f:
                f.write(p.read_bytes())
    write(narration / 'external-units.json', plan)
    canonical.write_bytes((narration / 'external-units.json').read_bytes())
    report = read(OLD / 'report.json')
    assembly = read(OLD / 'assembly.json')
    report['meta']['units_sha256'] = sha(canonical)
    assembly['manifest']['units_sha256'] = sha(canonical)
    report['meta']['direction_contract']['binding']['contract'] = plan['direction_contract']
    holds = {row['cue']:row for row in mapping['visual_holds']}
    for rows in (report['meta']['direction_contract']['dispositions'], assembly['manifest']['direction_dispositions']):
        assert len(rows) == 23
        for row in rows:
            hold = holds[row['cue']]
            assert row['kind'] == 'native_visual_hold'
            assert row['evidence']['path'] == str(R / 'source552/workspace' / hold['path'])
            row['evidence'] = dict(path=str(W / hold['path']), sha256=hold['sha256'])
            row['reason'] = f"Existing native visual hold retained at {hold['path']}:{hold['line']}. No samples or silence changed; no audio heard or checked."
    assert report['lines'] == read(OLD / 'report.json')['lines']
    assert assembly['outputs'] == read(OLD / 'assembly.json')['outputs']
    assert assembly['manifest']['units'] == read(OLD / 'assembly.json')['manifest']['units']
    write(narration / 'report.json', report)
    assembly['report_sha256'] = sha(narration / 'report.json')
    write(narration / 'assembly.json', assembly)
    results = {}
    for name, chapters, wanted in [('all', h.CHAPTERS, 105), ('boundary', ['09_boundary'], 9)]:
        cmd = [sys.executable, '-I', '-B', str(W / 'tools/narration_delivery.py'), '--topic', str(TOPIC), '--narration', str(narration), '--consumer', 'render', '--gain-db', '0']
        for chapter in chapters:
            cmd += ['--chapter-key', chapter]
        with (JOB / f'delivery-{name}.stdout').open('xb') as out, (JOB / f'delivery-{name}.stderr').open('xb') as err:
            subprocess.run(cmd, cwd=R/'repo', stdout=out, stderr=err, check=True, timeout=1800)
        result = read(JOB / f'delivery-{name}.stdout')
        assert result['verified_cues'] == 105 and result['selected_cues'] == wanted
        assert result['plan_mode'] == 'external-ui' and result['listening_acceptance'] is False
        results[name] = result
    for p in OLD.iterdir():
        if p.name not in ('report.json', 'assembly.json', 'external-units.json'):
            assert sha(p) == sha(narration / p.name)
    expected = dict(before)
    for p in (canonical, backup, W / MAPREL):
        expected[p.relative_to(SOURCE).as_posix()] = sha(p)
    assert snapshot(SOURCE) == expected and snapshot(OLD) == original
    write(JOB / 'delivery764.json', dict(wavs_byte_identical=105, report_sentence_rows_unchanged=True, phrase_timings_unchanged=True, delivery=results, source_after=expected, scope='Source and sample reconstruction only. Pictures, listening, fine-sync and independent acceptance remain open.'))


if __name__ == '__main__':
    main()
