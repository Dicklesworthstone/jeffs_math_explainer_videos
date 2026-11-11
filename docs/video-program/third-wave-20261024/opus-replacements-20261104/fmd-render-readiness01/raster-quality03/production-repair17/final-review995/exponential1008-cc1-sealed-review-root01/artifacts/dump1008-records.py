"""Read-only, run on CSS via ssh stdin, after the pictures1008 jobs exist:
  ssh css /usr/bin/python3 -I -B - < dump1008-records.py > records1008-NN.json
Prints one JSON object: each existing pictures1008 chapter record (with its file sha256 and the job receipt's
admitted/finished/exit), and recheck1008.json if present. Writes nothing on CSS."""
import hashlib
import json
from pathlib import Path

D = Path('/data/projects/jmev-css-render-worker-20261107')
KEYS = json.loads((D / 'jobs/css-exponential-build959/qualification.json').read_text())['chapters']


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def receipt(job):
    r = job / 'resource-receipt.json'
    if not r.is_file():
        return None
    v = json.loads(r.read_text())
    return {k: v.get(k) for k in ('admitted', 'finished', 'exit', 'resource_stop', 'error', 'inputs_sha256', 'final_inputs_sha256', 'command')}


out = {'chapters': {}, 'recheck': None}
for key in KEYS:
    job = D / 'jobs' / f'css-exponential-master-pictures1008-{key}'
    rec = job / 'pictures1008.json'
    out['chapters'][key] = {'job_exists': job.exists(), 'receipt': receipt(job) if job.exists() else None,
                            'record_sha256': sha(rec) if rec.is_file() else None,
                            'record': json.loads(rec.read_text()) if rec.is_file() else None}
rj = D / 'jobs/css-exponential-pictures1008-recheck'
rr = rj / 'recheck1008.json'
if rj.exists():
    out['recheck'] = {'receipt': receipt(rj), 'record_sha256': sha(rr) if rr.is_file() else None,
                      'record': json.loads(rr.read_text()) if rr.is_file() else None}
print(json.dumps(out))
