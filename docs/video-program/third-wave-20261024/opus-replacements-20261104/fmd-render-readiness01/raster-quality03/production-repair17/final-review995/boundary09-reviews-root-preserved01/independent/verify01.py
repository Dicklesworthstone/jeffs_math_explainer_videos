"""verify01.py: CC0's own check of CC3's local copy of the 09 scope02 extraction before viewing. Hashes only (no image
is decoded or processed): pictures.json at the sha CC3 reports from CSD; every picture and sheet file's sha256 equals
the record's entry for the same file name; counts; the record's scene-independent fields; the job receipt's success.

usage: python3 -I -B verify01.py COPY_DIR OUT_FILE
"""
import hashlib
import json
import sys
from pathlib import Path

COPY, OUT = Path(sys.argv[1]), Path(sys.argv[2])
lines, bad = [], []
say = lines.append


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def check(name, ok):
    say('%-4s %s' % ('ok' if ok else 'FAIL', name))
    if not ok:
        bad.append(name)


ch = COPY/'09_boundary'
rec = json.loads((ch/'pictures.json').read_text())
check('pictures.json sha256 = e2f1c855... (CC3: same on CSD before and after the copy)',
      sha(ch/'pictures.json') == 'e2f1c855ce0037da9e72ddce9d8a0f59fec5e784212a4f3e606b968862408411')
check('movie_sha256 = da4df9de... and frames 16504, full decode', rec['movie_sha256'].startswith('da4df9de') and
      rec['frames'] == 16504 and rec['full_decode'] is True)
pics = {Path(x['path']).name: x['sha256'] for x in rec['pictures']}
local = sorted(p.name for p in (ch/'pictures').iterdir())
check('%d pictures recorded, %d local, same names' % (len(pics), len(local)), sorted(pics) == local)
check('every local picture hashes to its record entry', all(sha(ch/'pictures'/n) == pics[n] for n in local))
sheets = {Path(x['path']).name: x for x in rec['sheets']}
local_s = sorted(p.name for p in (ch/'sheets').iterdir())
check('%d sheets recorded, %d local, same names' % (len(sheets), len(local_s)), sorted(sheets) == local_s)
check('every local sheet hashes to its record entry', all(sha(ch/'sheets'/n) == sheets[n]['sha256'] for n in local_s))
check('selected frames %d, sorted and distinct, one picture each' % len(rec['selected_frames']),
      rec['selected_frames'] == sorted(set(rec['selected_frames'])) and len(rec['selected_frames']) == len(pics))
r = json.loads((COPY/'tables'/'resource-receipt.json').read_text())
check('job receipt: admitted, finished, exit 0, inputs unchanged, no error or resource stop',
      bool(r.get('admitted') and r.get('finished') and r.get('exit') == 0 and r.get('inputs_sha256') == r.get('final_inputs_sha256')
           and not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit'))))
say('     receipt %s; record scope: %s' % (sha(COPY/'tables'/'resource-receipt.json')[:8], rec['scope']))
for name in ('decode.stderr', 'extract.stderr', 'probe.stderr'):
    check('%s is empty' % name, (ch/name).stat().st_size == 0)
say('RESULT: %s' % ('all checks passed' if not bad else 'FAILED: '+'; '.join(bad)))
with open(OUT, 'x', encoding='utf-8') as stream:
    stream.write('\n'.join(lines)+'\n')
print('\n'.join(lines))
