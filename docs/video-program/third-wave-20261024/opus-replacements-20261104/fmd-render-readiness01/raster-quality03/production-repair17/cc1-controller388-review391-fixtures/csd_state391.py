"""Read-only on CSD: staged 377/388 files, master/377 job folders, guard pins, and actual finished-job
receipt shapes and command.log last lines (the guard's own output vs the child's). Writes nothing."""
import hashlib
import json
import os
from pathlib import Path

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D / 'qualification01/production-repair17'


def sha(p):
    with open(p, 'rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


out = {'q': {}, 'jobs': {}, 'guard': {}}
for name in ('csd-final-verify377.py', 'csd-final-controller388.py', 'cc2-final-verify377-notes.md',
             'cc2-final-controller388-notes.md', 'final-verify377-inputs.json', 'final-pictures377-overview-inputs.json',
             'final-pictures377-native-inputs.json', 'final-pictures377-overview-ledger388.json',
             'csd-final-controller388.log', 'combined-master350-inputs.json', 'cc0-combined-master350.py'):
    p = Q / name
    out['q'][name] = dict(exists=p.exists(), is_symlink=p.is_symlink(),
                          sha256=sha(p) if p.is_file() else None, bytes=p.stat().st_size if p.is_file() else None)
out['q_controller_like'] = sorted(n for n in os.listdir(Q) if '388' in n or '377' in n or 'controller' in n)
for rel in ('guard_csd02.py',
            'inputs/repo/docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py',
            'inputs/repo/narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/run_bounded_phase_oliver_owner48g_target14g_20261102.py',
            'qualification01/owned-process-stop02.py'):
    p = D / rel
    out['guard'][rel.split('/')[-1]] = dict(exists=p.is_file(), sha256=sha(p) if p.is_file() else None,
                                            realpath_differs=str(p.resolve()) != str(p))
jobs = D / 'jobs'
names = sorted(os.listdir(jobs))
out['job_count'] = len(names)
out['master_and_377_jobs'] = {n: True for n in names if 'master350' in n or '377' in n}
out['master_exists'] = (jobs / 'csd-combined-master350').exists()
# the most recent finished guarded jobs: receipt key order, terminal fields, guard's own output never in command.log
rows = []
for n in names:
    r = jobs / n / 'resource-receipt.json'
    if not r.is_file():
        rows.append(dict(job=n, receipt=False))
        continue
    try:
        rec = json.loads(r.read_text())
    except ValueError as e:
        rows.append(dict(job=n, receipt='unparsable', error=repr(e)[:80]))
        continue
    row = dict(job=n, mtime=r.stat().st_mtime, keys=list(rec), admitted=rec.get('admitted'), exit=rec.get('exit'),
               finished=bool(rec.get('finished')), error=bool(rec.get('error')),
               stopped=rec.get('stopped_for_resource_limit'), inputs_equal=rec.get('inputs_sha256') == rec.get('final_inputs_sha256'),
               command_head=[str(x) for x in rec.get('command', [])[:3]], receipt_bytes=r.stat().st_size)
    log = jobs / n / 'command.log'
    if log.is_file():
        size = log.stat().st_size
        with open(log, 'rb') as f:
            f.seek(max(0, size - 4096))
            tail = f.read().decode(errors='replace').splitlines()
        tail = [x for x in tail if x.strip()]
        row.update(log_bytes=size, last_line=(tail[-1][:300] if tail else None),
                   last_is_json=bool(tail) and tail[-1].startswith('{'),
                   guard_line_in_log=any(x.startswith('{"exit":') or x.startswith('{"admitted":') for x in tail))
    rows.append(row)
rows.sort(key=lambda x: x.get('mtime', 0))
out['recent_jobs'] = rows[-14:]
out['not_finished'] = [x['job'] for x in rows if x.get('receipt') is not False and not x.get('finished')]
out['no_receipt'] = [x['job'] for x in rows if x.get('receipt') is False]
lock = D / 'expanded-native-lane.lock'
out['lane_lock'] = dict(exists=lock.exists())
u = os.statvfs(D)
out['free_gib'] = round(u.f_bavail * u.f_frsize / 1024**3, 2)
print(json.dumps(out, indent=1))
