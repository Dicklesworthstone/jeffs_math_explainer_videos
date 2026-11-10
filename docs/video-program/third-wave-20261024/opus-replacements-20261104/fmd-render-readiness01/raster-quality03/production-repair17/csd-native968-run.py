"""Prepared higher-worker recovery; never launch while864 is active.

Identical844 binary, source, audio, score and visual quality; no speed claim.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import signal

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
JOB = D/'jobs/csd-native-final968'

def sha(p):
    with p.open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def read(p):
    return json.loads(p.read_text())

def success(p):
    d = read(p)
    assert d.get('admitted') and d.get('finished') and d.get('exit') == 0, p
    assert d['inputs_sha256'] == d['final_inputs_sha256'], p
    assert not any(d.get(k) for k in ('resource_stop', 'stopped_for_resource_limit', 'error')), p
    return d

def main():
    base = Q/'csd-native844-base761.py'
    assert sha(base) == '32bf6d9e08c1f21f2445debfd46ba090e0374007e4da5516310cb122774099e5'
    spec = importlib.util.spec_from_file_location('owned864', base)
    h = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(h)
    h.own_group()
    h.STATE['job'] = JOB
    h.CRATE = D/'work/production-repo07'
    for sig in h.STOP_SIGNALS:
        signal.signal(sig, h.on_stop)
    receipt = read(JOB/'resource-receipt.json')
    assert receipt['admitted'] is True and 'finished' not in receipt and 'exit' not in receipt
    assert receipt['phase'] == 'native'
    assert receipt['policy'] == dict(admission_gib=10, stop_free_gib=8,
        phase_growth_gib=4, work_cap_gib=56, target_growth_cap_gib=20, sample_seconds=10)
    assert os.environ['CARGO_TARGET_DIR'] == str(D/'work')
    assert sorted(p.name for p in JOB.iterdir()) == ['command.log', 'resource-receipt.json']
    previous = D/'jobs/csd-native-final864'
    old = read(previous/'resource-receipt.json')
    assert old.get('finished') and old.get('exit') not in (None, 0) and not old.get('resource_stop')
    assert old['inputs_sha256'] == old['final_inputs_sha256']
    cleanup = read(previous/'native.cleanup.json')
    # Root stopped this exact owned group after diagnosing four-core raster saturation.
    # This is a preserved failed attempt, not a timeout or successful render.
    assert cleanup['step'] == 'native' and cleanup['reason'] == 'signal 15'
    assert cleanup['group'] == 782328 and cleanup['child_exit'] == -15 and old['exit'] == -9
    for name in ('csd-native-build844', 'csd-native-delivery849', 'csd-native-timeline852',
                 'csd-native-preview853', 'csd-native-pictures855'):
        success(D/'jobs'/name/'resource-receipt.json')
    binary = D/'jobs/csd-native-build844/complex_analysis_visual-844'
    assert sha(binary) == 'e75258731b1786ae3182ef9eb21a7903a9e6c0bed6242413335f7dd233955384'
    pcm = D/'jobs/csd-native-preview853/preview/soundtrack/09_boundary/soundtrack_0.wav'
    assert sha(pcm) == 'f7f8aa6042a9f05d1b25db4c789f7f6b44cb393445015c057f1e87d9b0c8df51'
    # Do not infer picture acceptance from the extractor's successful exit.
    review = read(Q/'csd-native864-picture-admission.json')
    assert review['preview_job'] == 'csd-native-preview853'
    assert review['extraction_job'] == 'csd-native-pictures855'
    assert review['all_selected_pictures_viewed'] is True
    assert review['concrete_defects'] == [] and review['independent_blockers'] == []
    assert review['safe_to_advance_to_final4k'] is True
    for key in ('root_review', 'independent_review', 'extraction_record'):
        item = review[key]
        assert sha(Path(item['path'])) == item['sha256']
    extraction = D/'jobs/csd-native-pictures855/pictures855.json'
    assert review['extraction_record']['path'] == str(extraction)
    pictures = read(extraction)
    movie = D/'jobs/csd-native-preview853/preview/09_boundary.mp4'
    assert pictures['movie'] == str(movie) and pictures['movie_sha256'] == sha(movie)
    assert pictures['full_decode'] is True and pictures['frames'] == 16504
    frames = pictures['selected_frames']
    assert len(frames) >= 814 and frames == sorted(set(frames))
    for item in pictures['pictures'] + pictures['sheets']:
        assert sha(Path(item['path'])) == item['sha256']
    root = read(Path(review['root_review']['path']))
    independent = read(Path(review['independent_review']['path']))
    for evidence in (root, independent):
        assert evidence['movie_sha256'] == pictures['movie_sha256']
        assert evidence['extraction_sha256'] == sha(extraction)
        assert evidence['source_sha256'] == 'cd1900aadd46090083729d9aa768e59e1551c42ab94e5bd055f520cb4eff5e6e'
        assert evidence['concrete_defects'] == []
        assert evidence['safe_to_advance_to_final4k'] is True
    assert root['viewed_frame_indices'] == frames
    assert root['viewed_sheets'] == [item['path'] for item in pictures['sheets']]
    assert independent['authored_repair'] is False
    assert independent['viewed_frame_indices']
    assert set(independent['viewed_frame_indices']) <= set(frames)
    stems = D/'jobs/csd-production-score21/score/stems'
    expected = {
        '09_boundary.music.f32.parts.tsv': 'db31e88404eb9af836c6575cf5b4178a292eaeceea283d717bf2a4a7166ffe2e',
        '09_boundary.music.f32.wav.part0000.wav': 'ea2207e2a92ea9751fc688878b0a781aeb2a44c79a5e0a3de78197005d508483',
        '09_boundary.music.f32.wav.part0001.wav': '492613e4cefe62bf730532c375a17fd8815e497e570d81f07b76c052989b0758',
        '09_boundary.music.f32.wav.part0002.wav': 'a57f58d245399bdbc8a615b9c3d85cefd1891449de2bccfa9a541dc506efb3de',
        'qa/09_boundary.json': '445daec7a3652361923c34c2a0e6e3e50b23a2b596d89f4744ccf3f350e6b58e',
    }
    for name, digest in expected.items():
        assert sha(stems/name) == digest, name
    cpus = sorted(os.sched_getaffinity(0))
    assert len(cpus) >= 16, 'Higher-worker recovery requires16 available CPUs'
    cores = set()
    for cpu in cpus[:16]:
        topology = Path('/sys/devices/system/cpu')/f'cpu{cpu}'/'topology'
        cores.add(((topology/'physical_package_id').read_text().strip(),
                   (topology/'core_id').read_text().strip()))
    assert len(cores) == 16, 'Do not count sibling logical CPUs as distinct cores'
    os.sched_setaffinity(0, cpus[:16])
    os.environ.update(PATH='/usr/bin:/bin', TMPDIR=str(D/'work/tmp'), XDG_CACHE_HOME=str(D/'work/cache'))
    args = [binary, 'render', '09_boundary', '--res', '3840x2160', '--fps', '60',
            '--threads', '16', '--narration', D/'jobs/csd-native-delivery849/narration',
            '--score', stems, '--out', JOB/'render', '--lint', '--crf', '16', '--preset', 'veryfast']
    h.run_step(JOB, 'native', args, 86400, None)

if __name__ == '__main__':
    main()
