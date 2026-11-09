"""Inspect all 15 joins in the actual encoded algebra master; ASR is provisional."""
from pathlib import Path
import hashlib
import json
import subprocess
import wave

R = Path('/Users/jemanuel/projects/jeffs_math_explainer_videos')
B = R / 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
J = B / 'algebra-final-joins15'
M = B / 'algebra-final-master14'
N = B / 'algebra-fixes-assemble03/narration'
MODEL = Path('/Users/jemanuel/.cache/franken_whisper/models/whisper/whisper-large-v3-turbo-f16-v1/ggml-large-v3-turbo.bin')
FW = Path('/opt/homebrew/bin/fw')


def sha(p):
    with Path(p).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def run():
    admitted = json.loads((J / 'resource-receipt.json').read_text())
    assert admitted['admitted'] and 'exit' not in admitted
    for name in ('algebra-final-master14', 'algebra-final-verify14'):
        receipt = json.loads((B / name / 'resource-receipt.json').read_text())
        assert receipt['exit'] == 0 and receipt['inputs_sha256'] == receipt['final_inputs_sha256']
        assert not receipt.get('resource_stop')
    assert sha(MODEL) == '1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69'
    summary = json.loads((M / 'master-summary.json').read_text())
    movie = Path(summary['outputs']['master']['path'])
    assert sha(movie) == summary['outputs']['master']['sha256']
    script_path = B / 'algebra-fixes-inputs02/workspace/explainer_corpus/abstract_algebra_expanded/script.tsv'
    script = dict(line.split('\t', 1) for line in script_path.read_text().splitlines()
                  if line.strip() and not line.startswith('#'))
    chapters = []
    offset = 0
    for chapter in summary['chapters']:
        rows = [line.split('\t') for line in (M / 'final' / (chapter + '.timeline.tsv')).read_text().splitlines()]
        frames = next(int(row[1]) for row in rows if row[0] == 'frames')
        cues = []
        for row in rows:
            if row[0] != 'line':
                continue
            source = Path(row[4])
            assert sha(source) == sha(N / (row[1] + '.wav'))
            with wave.open(str(source)) as wav:
                duration = wav.getnframes() / wav.getframerate()
            cues.append(dict(cue=row[1], start=(offset + int(row[2])) / 60,
                             duration=duration, script=script[row[1]]))
        chapters.append(dict(chapter=chapter, start=offset / 60, cues=cues))
        offset += frames
    assert offset == 251862 and len(chapters) == 16
    results = []
    for i, (left, right) in enumerate(zip(chapters, chapters[1:]), 1):
        before, after = left['cues'][-1], right['cues'][0]
        start = max(0, before['start'] - .25)
        end = min(offset / 60, after['start'] + after['duration'] + .25)
        stem = J / f'join-{i:02d}'
        audio = stem.with_suffix('.wav')
        spectrum = stem.with_suffix('.png')
        with stem.with_suffix('.extract.log').open('x') as log:
            subprocess.run(['ffmpeg', '-nostdin', '-n', '-v', 'error', '-ss', str(start), '-i', str(movie),
                            '-t', str(end-start), '-vn', '-ar', '16000', '-ac', '1', '-c:a', 'pcm_s16le', str(audio)],
                           stdout=log, stderr=subprocess.STDOUT, check=True, timeout=600)
        with wave.open(str(audio)) as wav:
            assert (wav.getframerate(), wav.getnchannels(), wav.getsampwidth()) == (16000, 1, 2)
            assert abs(wav.getnframes()/16000 - (end-start)) < .05
        with stem.with_suffix('.spectrum.log').open('x') as log:
            subprocess.run(['ffmpeg', '-nostdin', '-n', '-v', 'error', '-i', str(audio), '-lavfi',
                            'showspectrumpic=s=1600x600:legend=1:scale=log:color=intensity', '-frames:v', '1', str(spectrum)],
                           stdout=log, stderr=subprocess.STDOUT, check=True, timeout=600)
        transcript = stem.with_suffix('.json')
        with transcript.open('x') as out, stem.with_suffix('.asr.log').open('x') as err:
            subprocess.run([str(FW), 'transcribe', '--input', str(audio), '--model', str(MODEL),
                            '--language', 'en', '--no-diarize', '--no-persist', '--json',
                            '--max-segment-length', '1', '--split-on-word', '--threads', '4', '--timeout', '1800'],
                           stdout=out, stderr=err, check=True, timeout=1900)
        parsed = json.loads(transcript.read_text())
        assert isinstance(parsed, dict) and parsed
        results.append(dict(index=i, left=left['chapter'], right=right['chapter'], start=start, end=end,
                            chapter_join=right['start'], before=before, after=after,
                            scheduled_gap_seconds=after['start']-before['start']-before['duration'],
                            files=[dict(path=str(p), sha256=sha(p)) for p in (audio, spectrum, transcript)],
                            transcript_read=False, spectrogram_viewed=False))
        (J / 'progress.json').write_text(json.dumps(results, indent=2))
    assert len(results) == 15
    (J / 'joins.json').write_text(json.dumps(dict(master_sha256=sha(movie), joins=results,
        listening_accepted=False, independent_av_accepted=False,
        scope='Actual encoded-master windows spanning complete last/first cues at every chapter join. '
              'Provisional word-level ASR and spectrograms require actual reading; no perceptual verdict.'), indent=2))


if __name__ == '__main__':
    run()
