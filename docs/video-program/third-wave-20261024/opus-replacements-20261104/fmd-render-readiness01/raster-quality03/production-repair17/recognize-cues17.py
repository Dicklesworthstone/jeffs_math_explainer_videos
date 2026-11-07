"""Recognize preserved full takes under the existing guard; never synthesize or select."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

plan = json.loads(Path(sys.argv[1]).read_text())
job = Path(plan['job_path'])
receipt = json.loads((job / 'resource-receipt.json').read_text())
assert receipt['admitted'] and 'exit' not in receipt
for row in plan['takes']:
    source = Path(row['path'])
    assert hashlib.sha256(source.read_bytes()).hexdigest() == row['sha256']
    args = [plan['fw'], 'transcribe', '--input', str(source), '--backend', 'whisper-cpp',
            '--model', plan['model'], '--language', 'en', '--no-diarize', '--no-persist',
            '--split-on-word', '--max-segment-length', '1', '--threads', '4',
            '--timeout', '600', '--json']
    with (job / (row['id'] + '.json')).open('xb') as output, (job / (row['id'] + '.stderr')).open('xb') as error:
        result = subprocess.run(args, stdout=output, stderr=error, timeout=660)
    assert result.returncode == 0, (row['id'], result.returncode)
    parsed = json.loads((job / (row['id'] + '.json')).read_text())
    assert parsed, row['id']
    print('Recognition retained:', row['id'], flush=True)
print('ASR evidence only. No listening, pronunciation or fine synchronization acceptance.', flush=True)
