"""Guarded source-only rebind of retained Linux313 narration to repaired native844. Prepared, not run."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import os
import signal

R = Path('/data/projects/jmev-stroke-probe-20261106')
Q = R / 'qualification01/production-repair17'
SOURCE = R / 'work/native-boundary844'
W = SOURCE / 'workspace'
TOPIC = W / 'explainer_corpus/complex_analysis_visual'
OLD = R / 'jobs/csd-review-repair-rebind313/narration'
JOB = R / 'jobs/csd-native-delivery849'
MAPREL = 'explainer_corpus/complex_analysis_visual/docs/narration-direction-map-frederick-review849.json'


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
    assert sys.argv[1:] == ['--run']
    helper = Q / 'csd-native844-base761.py'
    assert sha(helper) == '32bf6d9e08c1f21f2445debfd46ba090e0374007e4da5516310cb122774099e5'
    spec = importlib.util.spec_from_file_location('owned849', helper)
    h = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(h)
    h.own_group()
    h.STATE['job'] = JOB
    h.CRATE = R / 'work/production-repo07'
    for sig in h.STOP_SIGNALS:
        signal.signal(sig, h.on_stop)
    receipt = read(JOB / 'resource-receipt.json')
    assert receipt['admitted'] is True and 'finished' not in receipt and 'exit' not in receipt
    assert receipt['phase'] == 'native'
    assert receipt['policy'] == dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4,
        work_cap_gib=56, target_growth_cap_gib=20, sample_seconds=10)
    assert os.environ['CARGO_TARGET_DIR'] == str(R / 'work')
    assert sorted(p.name for p in JOB.iterdir()) == ['command.log', 'resource-receipt.json']
    build = R / 'jobs/csd-native-build844'
    success(build / 'resource-receipt.json')
    success(OLD.parent / 'resource-receipt.json')
    qualification = read(build / 'qualification.json')
    assert qualification['tests_passed'] == 235 and qualification['clippy_passed'] is True
    assert qualification['explicit_native_plate_fade_probe_passed'] is True
    assert qualification['chapters'] == h.CHAPTERS
    assert qualification['facts_sha256'] == h.FACTS_SHA256
    assert qualification['binary'] == str(build / 'complex_analysis_visual-844')
    assert sha(build / 'complex_analysis_visual-844') == qualification['binary_sha256']
    assert list(h.listing(SOURCE)) == qualification['source_listing']
    before = snapshot(SOURCE)
    original = snapshot(OLD)
    assert len(list(OLD.glob('*.wav'))) == 105
    assert sha(OLD / 'external-units.json') == '9308c2b230a7cafd933dc826511bc8111748e9355ab9e445d3f7f89c81511c39'
    assert sha(OLD / 'report.json') == '69cb32ee3eb9cf83c5d8b18da8bda7682f4a55d1b91e21ad0b543ca73e25af85'
    assert sha(OLD / 'assembly.json') == '7a9dcf43f1581bcb7888c396839e33781d1ef56f515c44b30a1551f9e4eb50b6'
    mapfile = Q / 'csd-native849-map.json'
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
    backup = TOPIC / 'narration_units.before-linux849.json'
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
            assert row['evidence']['path'] == str(R / 'work/review-repair313/workspace' / hold['path'])
            row['evidence'] = dict(path=str(W / hold['path']), sha256=hold['sha256'])
            row['reason'] = f"Existing native visual hold retained at {hold['path']}:{hold['line']}. No samples or silence changed; no audio heard or checked."
    assert report['lines'] == read(OLD / 'report.json')['lines']
    assert assembly['outputs'] == read(OLD / 'assembly.json')['outputs']
    assert assembly['manifest']['units'] == read(OLD / 'assembly.json')['manifest']['units']
    write(narration / 'report.json', report)
    assembly['report_sha256'] = sha(narration / 'report.json')
    write(narration / 'assembly.json', assembly)
    h.run_step(JOB, 'coherent', [sys.executable, '-I', '-B', W / 'tools/coherent_narration.py',
        'verify', narration, '--script', TOPIC / 'script.tsv', '--units', narration / 'external-units.json',
        '--require-directions'], 1800, None)
    results = {}
    for name, chapters, wanted in [('all', h.CHAPTERS, 105), ('boundary', ['09_boundary'], 9)]:
        cmd = [sys.executable, '-I', '-B', str(W / 'tools/narration_delivery.py'), '--topic', str(TOPIC), '--narration', str(narration), '--consumer', 'render', '--gain-db', '0']
        for chapter in chapters:
            cmd += ['--chapter-key', chapter]
        h.run_step(JOB, f'delivery-{name}', cmd, 1800, None)
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
    write(JOB / 'delivery849.json', dict(wavs_byte_identical=105, report_sentence_rows_unchanged=True, phrase_timings_unchanged=True, delivery=results, source_after=expected, scope='Source and sample reconstruction only. Pictures, listening, fine-sync and independent acceptance remain open.'))


if __name__ == '__main__':
    main()
