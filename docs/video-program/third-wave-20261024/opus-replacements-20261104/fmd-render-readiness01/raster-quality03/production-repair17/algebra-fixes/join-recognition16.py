"""Recheck disputed exported join windows without word-split decoding.

Same-model diagnostic, not independent recognition or listening acceptance.
"""
import hashlib
import json
from pathlib import Path
import subprocess

R = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
B = R / 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
Q = R / 'docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17'
A = Q / 'algebra-fixes'
OLD = B / 'algebra-final-joins15'
JOB = B / 'algebra-join-recognition16'
MODEL = Path('/Users/jemanuel/.cache/franken_whisper/models/whisper/whisper-large-v3-turbo-f16-v1/ggml-large-v3-turbo.bin')


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    receipt = json.loads((OLD / 'resource-receipt.json').read_text())
    assert receipt['exit'] == 0 and not receipt.get('resource_stop')
    assert receipt['inputs_sha256'] == receipt['final_inputs_sha256']
    if not JOB.exists():
        inherited = json.loads((A / 'final-joins15-inputs.json').read_text())['files']
        extra = [Path(__file__).resolve(), OLD / 'resource-receipt.json']
        extra += [OLD / f'join-{i:02}.wav' for i in (1, 3, 6, 9, 10)]
        files = {row['path']: row for row in inherited}
        for path in extra:
            files[str(path)] = dict(path=str(path), sha256=sha(path))
        for row in files.values():
            assert sha(row['path']) == row['sha256'], row['path']
        manifest = A / 'join-recognition16-inputs.json'
        with manifest.open('x') as out:
            json.dump(dict(files=list(files.values())), out, indent=2)
        with (A / 'join-recognition16-guard.log').open('x') as log:
            subprocess.run(['python3', '-I', '-B', str(Q / 'guard_owner80g_target20g.py'),
                            '--phase', 'native', '--job', JOB.name, '--inputs', str(manifest),
                            '--', 'python3', '-I', '-B', str(Path(__file__).resolve())],
                           cwd=R, stdout=log, stderr=subprocess.STDOUT, check=True)
        return
    admitted = json.loads((JOB / 'resource-receipt.json').read_text())
    assert admitted['admitted'] and 'exit' not in admitted
    for i in (1, 3, 6, 9, 10):
        with (JOB / f'join-{i:02}.json').open('x') as out, (JOB / f'join-{i:02}.stderr').open('x') as err:
            subprocess.run(['/opt/homebrew/bin/fw', 'transcribe', '--input', str(OLD / f'join-{i:02}.wav'),
                            '--model', str(MODEL), '--language', 'en', '--no-diarize', '--no-persist',
                            '--json', '--threads', '4', '--timeout', '1800'],
                           cwd=R, stdout=out, stderr=err, check=True, timeout=1900)


if __name__ == '__main__':
    main()
