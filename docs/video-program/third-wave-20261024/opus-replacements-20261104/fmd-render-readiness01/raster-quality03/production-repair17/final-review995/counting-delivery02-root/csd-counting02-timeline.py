"""Chapter11 repaired native timeline; unchanged schedules required before rendering."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import signal
import sys

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
JOB = D/'jobs/csd-native-counting-timeline02'
BUILD = D/'jobs/csd-native-counting-build02'
DELIVERY = D/'jobs/csd-native-counting-delivery02'
EXPECTED = {'11_counting': (11932, '29a3afd345fa3a376e5c163d5bf5e01267a866e2305bdd27c6413a5bab46807d', '/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-rebind269/narration')}


def read(p):
    return json.loads(p.read_text())


def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def success(p):
    r = read(p)
    assert r.get('admitted') and r.get('finished') and r.get('exit') == 0, p
    assert r['inputs_sha256'] == r['final_inputs_sha256'], p
    assert not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), p


def main():
    assert sys.argv[1:] == ['--run']
    base = Q/'csd-native844-base761.py'
    assert sha(base) == '32bf6d9e08c1f21f2445debfd46ba090e0374007e4da5516310cb122774099e5'
    spec = importlib.util.spec_from_file_location('owned_combined_timeline', base)
    h = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(h)
    h.own_group()
    h.STATE['job'] = JOB
    h.CRATE = D/'work/production-repo07'
    for sig in h.STOP_SIGNALS:
        signal.signal(sig, h.on_stop)
    r = read(JOB/'resource-receipt.json')
    assert r.get('admitted') and 'finished' not in r and 'exit' not in r
    assert r['phase'] == 'native' and r['policy'] == dict(admission_gib=10, stop_free_gib=8,
        phase_growth_gib=4, work_cap_gib=56, target_growth_cap_gib=20, sample_seconds=10)
    assert os.environ['CARGO_TARGET_DIR'] == str(D/'work')
    assert sorted(p.name for p in JOB.iterdir()) == ['command.log', 'resource-receipt.json']
    success(BUILD/'resource-receipt.json')
    success(DELIVERY/'resource-receipt.json')
    qualification = read(BUILD/'qualification.json')
    binary = BUILD/'complex_analysis_visual-counting02'
    assert qualification['tests_passed'] == 250 and qualification['clippy_passed']
    assert sha(binary) == qualification['binary_sha256']
    delivery = read(DELIVERY/'delivery-counting02.json')
    assert delivery['wavs_byte_identical'] == 105
    assert delivery['delivery']['repaired']['selected_cues'] == 28
    os.sched_setaffinity(0, sorted(os.sched_getaffinity(0))[:4])
    os.environ.update(PATH='/usr/bin:/bin', TMPDIR=str(D/'work/tmp'), XDG_CACHE_HOME=str(D/'work/cache'))
    out = JOB/'timeline-output'
    checks = {}
    for chapter, (frames, digest, old_dir) in EXPECTED.items():
        h.run_step(JOB, chapter, [binary, 'render', chapter, '--fps', '60', '--threads', '4',
            '--narration', DELIVERY/'narration', '--out', out, '--timeline-only'], 14400, None)
        path = out/(chapter+'.timeline.tsv')
        raw = path.read_text()
        lines = raw.splitlines()
        assert [x for x in lines if x.startswith('frames\t')] == ['frames\t'+str(frames)]
        normalized = []
        for line in raw.split('\n'):
            cols = line.split('\t')
            if cols[0] == 'line':
                assert cols[-1].startswith(str(DELIVERY/'narration')+'/'), cols
                cols[-1] = old_dir+'/'+cols[-1].rsplit('/', 1)[1]
            normalized.append('\t'.join(cols))
        got = hashlib.sha256('\n'.join(normalized).encode()).hexdigest()
        assert got == digest, (chapter, got, digest)
        checks[chapter] = dict(frames=frames, actual_sha256=sha(path), normalized_sha256=got)
    with (JOB/'comparison-counting02.json').open('x') as f:
        json.dump(dict(chapters=checks, scope='Exact full normalized chapter11 timeline. No picture, motion, listening or publication acceptance.'), f, indent=2)


if __name__ == '__main__':
    main()
