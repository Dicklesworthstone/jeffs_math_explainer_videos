"""Reassemble retained Oliver audio on mac-mini-max after the source-map rebind (bead 87do.63).

Root's S/assemble01.py with the remote job, manifest and output paths. The narration is written inside the job folder,
so the guard counts it. Launch only through remote_audio_after_build01.py. No narration is generated or time-processed.
"""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys

HERE = Path(__file__).resolve()
R = HERE.parents[5]
EL = R / 'narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104'
S = EL / 'final-picture-repair01'
O = EL / 'remote-mac-mini-max01'
JOB = O / 'jobs/logarithm-remote-assemble01'
C = S / 'workspace/explainer_corpus/logarithm_mysteries'
TOOL = S / 'workspace/tools/coherent_narration.py'
OLD = EL / 'production-audio-stage09-01/prepared02/narration'
OUT = JOB / 'narration01'


def sha(path):
    with path.open('rb') as handle:
        return hashlib.file_digest(handle, 'sha256').hexdigest()


def main():
    receipt = json.loads((JOB / 'resource-receipt.json').read_text())
    assert receipt.get('admitted') is True and 'exit' not in receipt
    assert receipt['command'] == [sys.executable, '-I', '-B', str(HERE)]
    manifest = json.loads((O / 'assemble-inputs01.json').read_text())
    for row in manifest['files']:
        assert sha(Path(row['path'])) == row['sha256'], row['path']
    assert (S / 'integration.json').is_file() and (S / 'holds-bound01.json').is_file()
    assert not OUT.exists(), 'Preserve any earlier assembly'
    common = ['--script', str(C / 'script.tsv'), '--units', str(C / 'narration_units.json'),
              '--require-directions']
    subprocess.run([sys.executable, '-I', '-B', str(TOOL), 'assemble', *common,
                    '--spans', str(S / 'spans01.json'), '--out', str(OUT)],
                   cwd=R, check=True, timeout=900)
    cues = [line.split('\t', 1)[0] for line in (C / 'script.tsv').read_text().splitlines()
            if line and not line.startswith('#')]
    assert len(cues) == len(set(cues)) == 80
    assert {p.stem for p in OUT.glob('*.wav')} == set(cues)
    hashes = {}
    for cue in cues:
        digest = sha(OLD / (cue + '.wav'))
        assert sha(OUT / (cue + '.wav')) == digest, cue
        hashes[cue] = digest
    timings = OLD / 'phrase-timings.json'
    assert not (OUT / timings.name).exists()
    shutil.copy2(timings, OUT / timings.name)
    assert sha(timings) == sha(OUT / timings.name)
    subprocess.run([sys.executable, '-I', '-B', str(TOOL), 'verify', str(OUT), *common],
                   cwd=R, check=True, timeout=900)
    with (JOB / 'audio-comparison.json').open('x') as handle:
        json.dump({'cues': 80, 'wav_sha256': hashes, 'phrase_timings_sha256': sha(timings),
                   'all_full_wavs_byte_identical': True, 'all_phrase_timings_byte_identical': True,
                   'scope': 'Sample reconstruction and source-direction verification only; '
                            'recognition timing remains provisional. No listening or fine-sync acceptance.'},
                  handle, indent=2)


if __name__ == '__main__':
    main()
