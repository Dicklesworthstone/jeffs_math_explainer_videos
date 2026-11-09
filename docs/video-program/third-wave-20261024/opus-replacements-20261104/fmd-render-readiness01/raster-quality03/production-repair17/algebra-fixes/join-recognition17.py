"""Diagnose ASR window-edge omissions on sample-exact exported cue excerpts."""
import hashlib
import json
from pathlib import Path
import subprocess
import wave

R = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
B = R / 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
Q = R / 'docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17'
A = Q / 'algebra-fixes'
OLD = B / 'algebra-final-joins15'
JOB = B / 'algebra-join-recognition17'
MODEL = Path('/Users/jemanuel/.cache/franken_whisper/models/whisper/whisper-large-v3-turbo-f16-v1/ggml-large-v3-turbo.bin')


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def main():
    receipt_path = B / 'algebra-join-recognition16/resource-receipt.json'
    receipt = json.loads(receipt_path.read_text())
    assert receipt['exit'] == 0 and not receipt.get('resource_stop')
    assert receipt['inputs_sha256'] == receipt['final_inputs_sha256']
    if not JOB.exists():
        inherited = json.loads((A / 'join-recognition16-inputs.json').read_text())['files']
        files = {row['path']: row for row in inherited}
        for path in [Path(__file__).resolve(), receipt_path, OLD / 'joins.json']:
            files[str(path)] = dict(path=str(path), sha256=sha(path))
        for row in files.values():
            assert sha(row['path']) == row['sha256'], row['path']
        manifest = A / 'join-recognition17-inputs.json'
        with manifest.open('x') as out:
            json.dump(dict(files=list(files.values())), out, indent=2)
        with (A / 'join-recognition17-guard.log').open('x') as log:
            subprocess.run(['python3', '-I', '-B', str(Q / 'guard_owner80g_target20g.py'),
                            '--phase', 'native', '--job', JOB.name, '--inputs', str(manifest),
                            '--', 'python3', '-I', '-B', str(Path(__file__).resolve())],
                           cwd=R, stdout=log, stderr=subprocess.STDOUT, check=True)
        return
    admitted = json.loads((JOB / 'resource-receipt.json').read_text())
    assert admitted['admitted'] and 'exit' not in admitted
    joins = json.loads((OLD / 'joins.json').read_text())['joins']
    evidence = []
    for i in (1, 3, 6, 9, 10):
        row = joins[i-1]
        # Full first cue, including a quarter second of context either side.
        # Join10 instead isolates the disputed phrase within the preceding cue.
        start = row['after']['start'] - row['start'] - .25
        end = start + row['after']['duration'] + .5
        if i == 10:
            start, end = 7.5, 18.5
        source = OLD / f'join-{i:02}.wav'
        audio = JOB / f'cue-{i:02}.wav'
        with wave.open(str(source)) as wav:
            assert (wav.getframerate(), wav.getnchannels(), wav.getsampwidth()) == (16000, 1, 2)
            first = max(0, round(start * 16000))
            last = min(wav.getnframes(), round(end * 16000))
            wav.setpos(first)
            data = wav.readframes(last-first)
        assert last > first and (last-first)/16000 < 30
        assert not audio.exists()
        with wave.open(str(audio), 'wb') as out:
            out.setparams((1, 2, 16000, 0, 'NONE', 'not compressed'))
            out.writeframes(data)
        evidence.append(dict(join=i, source=str(source), source_sha256=sha(source),
                             first_sample=first, end_sample_exclusive=last, sha256=sha(audio)))
        with (JOB / f'cue-{i:02}.json').open('x') as out, (JOB / f'cue-{i:02}.stderr').open('x') as err:
            subprocess.run(['/opt/homebrew/bin/fw', 'transcribe', '--input', str(audio),
                            '--model', str(MODEL), '--language', 'en', '--no-diarize', '--no-persist',
                            '--json', '--threads', '4', '--timeout', '1800'],
                           cwd=R, stdout=out, stderr=err, check=True, timeout=1900)
    with (JOB / 'excerpts.json').open('x') as out:
        json.dump(dict(excerpts=evidence, listening_accepted=False,
                       scope='Same-model diagnostic with different window context; no speech verdict.'), out, indent=2)


if __name__ == '__main__':
    main()
