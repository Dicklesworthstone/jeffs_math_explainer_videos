"""Freeze actual csd master/runtime, launch once under guard, verify receipts.
Neither successful extraction nor this postcheck accepts the resulting pictures.
"""
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys


def load(name):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(name))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


V = load('final-verify06.py')
RUNTIME = load('launch-pictures01.py')
K, S, B = V.K, V.S, V.B


def postcheck():
    terminal = S.success(V.NAME)
    assert terminal['command'] == V.child()
    assert S.sha(V.MANIFEST) == terminal['inputs_sha256']
    B.collect(V.MANIFEST, [])
    assert (V.JOB / 'command.log').read_text().splitlines()[-1] == 'FINAL VERIFY technical checks passed; actual final picture review remains.'
    result = S.read(V.JOB / 'verification.json')
    cfg = V.plan()
    for key, value in cfg.items():
        assert result[key] == value, key
    assert result['pictures_viewed'] is False and result['binary_sha256'] == S.BINARY_SHA
    assert result['sha256'] == cfg['outputs']['master']['sha256']
    assert len(result['pictures']) == len(cfg['samples'])
    for sample, picture in zip(cfg['samples'], result['pictures']):
        assert all(picture[key] == value for key, value in sample.items())
        assert S.sha(Path(picture['path'])) == picture['sha256']
    expected = [str(V.JOB / f'sheet-{i+1:03d}.jpg') for i in range((len(cfg['samples']) + 3) // 4)]
    assert result['sheets'] == expected
    assert result['sheet_sha256'] == {p: S.sha(Path(p)) for p in expected}
    # Recompute the complete intended sampling set and inspect actual PNG
    # headers and bytes. Extraction success is still not visual acceptance.
    ends = {c['chapter']: c['start_frame'] + c['frames'] for c in cfg['chapters']}
    frames = {}
    for cue in cfg['placements']:
        for fraction in (0.25, 0.5, 0.9):
            frame = min(ends[cue['chapter']] - 1,
                        cue['start_frame'] + round(cue['duration_s'] * fraction * 60))
            frames.setdefault(frame, []).append(f"{cue['cue']} at {fraction:.0%}: native colour/detail review")
    details = result['native_detail_pictures']
    assert result['native_detail_viewed'] is False
    assert [row['frame'] for row in details] == sorted(frames)
    for row in details:
        path = V.JOB / 'native-detail' / f"frame-{row['frame']:07}.png"
        assert row['path'] == str(path) and row['reasons'] == frames[row['frame']]
        assert row['viewed'] is False and row['sha256'] == S.sha(path)
        header = path.read_bytes()[:24]
        assert header[:8] == b'\x89PNG\r\n\x1a\n' and header[12:16] == b'IHDR'
        assert (int.from_bytes(header[16:20], 'big'), int.from_bytes(header[20:24], 'big')) == (3840, 2160)
        assert (row['width'], row['height']) == (3840, 2160)
    shared = V.shared()
    strict = (V.JOB / 'strict-video-verification.log').read_text()
    assert strict.splitlines()[-1] == 'VERIFY PASS'
    assert result['full_decode'] == shared.decode_lines(strict)
    assert shared.software_decode_clean(result['full_decode'])
    assert result['encoded_loudness'] == shared.loudness_summary((V.JOB / 'encoded-loudness.log').read_text())
    assert float(result['encoded_loudness']['true_peak_dbtp']) <= -1.0
    screen = S.read(V.JOB / 'caption-screen.json')
    assert screen == result['caption_screen'] and screen['srt_sha256'] == cfg['outputs']['srt']['sha256']
    return {'master_sha256': result['sha256'], 'frames': cfg['frames'], 'pictures': len(result['pictures']),
            'sheets': len(expected), 'native_detail_frames': len(details),
            'verification_sha256': S.sha(V.JOB / 'verification.json'),
            'pictures_viewed': False, 'scope': 'Revalidated technical outputs only; actual final picture review and listening/fine-sync/independent AV remain.'}


def launch():
    assert Path(__file__).resolve().parent == S.ROOT / 'qualification01'
    assert not V.JOB.exists() and not V.MANIFEST.exists(), 'Never duplicate verification'
    cfg = V.plan()
    shared = V.shared()
    environment, runtime = RUNTIME.pillow_runtime()
    extra = [Path(__file__).resolve(), V.ME, K.ME, K.MANIFEST, V.SHARED,
             V.REPO / shared.STRICT_REL, V.REPO / shared.SCREEN_REL, V.SCRIPT,
             Path(RUNTIME.__file__).resolve(), K.JOB / 'resource-receipt.json',
             K.JOB / 'master-summary.json', K.JOB / 'command.log', K.JOB / 'staged-inputs.json',
             K.JOB / 'master-native.log', K.JOB / 'captions-native.log', *K.OUTPUTS.values()]
    extra.extend(environment)
    for sources in cfg['layout_sources'].values():
        extra.extend(Path(p) for p in sources)
    for chapter in S.CHAPTERS:
        extra.extend(K.OUT / (chapter + suffix) for suffix in ('.mp4', '.timeline.tsv', '.cues.tsv', '.cuelog.tsv'))
    for tool in ('ffmpeg', 'ffprobe', 'bash'):
        path = shutil.which(tool)
        assert path, tool
        extra.append(Path(path).resolve())
    extra.append(Path(sys.executable).resolve())
    runtime['files_sha256'] = {str(p): S.sha(p) for p in environment}
    runtime_path = S.ROOT / 'qualification01/final-verify-runtime01.json'
    with runtime_path.open('x') as stream:
        json.dump(runtime, stream, indent=2)
    extra.append(runtime_path)
    frozen = B.collect(K.MANIFEST, extra)
    assert V.plan() == cfg
    with V.MANIFEST.open('x') as stream:
        json.dump(frozen, stream, indent=2)
    subprocess.run(['/usr/bin/python3', '-I', '-B', str(S.ROOT / 'guard_csd02.py'), '--phase', 'native',
                    '--job', V.NAME, '--inputs', str(V.MANIFEST), '--', *V.child()], cwd=S.ROOT, check=True)
    result = postcheck()
    with (S.ROOT / 'qualification01/final-verification-postcheck01.json').open('x') as stream:
        json.dump(result, stream, indent=2)
    print(json.dumps(result), flush=True)


if __name__ == '__main__':
    if sys.argv[1:] == ['--run']:
        launch()
    elif sys.argv[1:] == ['--postcheck']:
        print(json.dumps(postcheck(), indent=2))
    else:
        raise SystemExit('Explicit --run or --postcheck only')
