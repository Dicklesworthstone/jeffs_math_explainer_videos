"""csd-native-final-chain1006.py (CC0, 2026-11-11): the waiting final-chain controller for stage 1006, master
1006 and verify 1007 (owner via root: stage and run the waiting final-chain controllers once when their
prerequisites exist; root owns review acceptance, admissions and Beads). Each step runs once, in order; the
first refusal or failure stops the chain. Nothing is retried. This controller writes no review, admission or
acceptance and claims none; the helpers it runs are pinned by sha256 and do their own checks.

1. Stage, one chapter at a time, in the order the chapters' final review records become ready. A chapter is
   ready when root's record Q/csd-native-final4k-review1006-<chapter with dashes>-root.json exists and keeps
   its bytes for STABLE seconds. Its review_route names the worker records it needs, which must exist and stay
   stable in the same way:
     no review_route (the original rule)  -> ...-independent.json
     a route naming 'independent'         -> ...-independent.json
     a route naming 'author'              -> ...-author.json
   Then: stage03 stage --chapter CH --job csd-native-stage1006-NN-a1 --root-review R [--independent-review I]
   [--author-review A]. stage03 checks the records against the actual final movie and final extraction
   (final_review_checks) and runs the guarded materializer.
2. stage03 compose of the four stages -> Q/csd-native-stage1006-composed-master-manifest.json.
3. master check, then master launch --wait-hours 1 on that manifest (job csd-combined-master1006).
4. verify check, launch-verify (job csd-final-verify1007), launch-pictures twice (overview, then native).

The final picture, motion, audio, caption and publication reviews stay open; every receipt is technical.

usage: /usr/bin/python3 -I -B csd-native-final-chain1006.py   (no arguments)
"""
import hashlib
import json
import subprocess
import sys
import time
from pathlib import Path

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
STAGE, STAGE_SHA = Q/'csd-native-repair-stage03.py', 'f3172a33592cc9cb68cc7a414b58cb1d500188ab7947ecace61ee91afba8183a'
MASTER, MASTER_SHA = Q/'combined-master-repaired01.py', '7443bd91e3f5d866eed40d6dbfbb4544dfdf235f1c4a7d30508d8108eae4a352'      # filled after K (gate04 -> master)
VERIFY, VERIFY_SHA = Q/'csd-final-verify-repaired02.py', '78d8dda0cd044ac937c1e07eb018a28438d09174e3da5620e54ff6ae87aafca6'     # filled after K (master -> verifier)
CHAPTERS = ('01_the_bump', '08_loop_sums', '09_boundary', '11_counting')
VERSION = '1006'
MANIFEST = Q/('csd-native-stage%s-composed-master-manifest.json' % VERSION)
POLL, STABLE, LIMIT = 60, 120, 96*3600
PYTHON = '/usr/bin/python3'


def unresolved():
    return [n for n in ('MASTER_SHA', 'VERIFY_SHA') if globals()[n] is None]


def say(text):
    print(time.strftime('%Y-%m-%dT%H:%M:%SZ', time.gmtime()), text, flush=True)


def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def record(chapter, who):
    return Q/('csd-native-final4k-review%s-%s-%s.json' % (VERSION, chapter.replace('_', '-'), who))


def stage_job(chapter):
    return 'csd-native-stage%s-%s-a1' % (VERSION, chapter[:2])


def pinned():
    for path, digest in ((STAGE, STAGE_SHA), (MASTER, MASTER_SHA), (VERIFY, VERIFY_SHA)):
        assert sha(path) == digest, 'changed: %s' % path


def stable(path, seen):
    """True once path has kept the same bytes for STABLE seconds (seen holds first sighting per path)."""
    if not path.is_file():
        seen.pop(path, None)
        return False
    data, now = path.read_bytes(), time.monotonic()
    if path not in seen or seen[path][0] != data:
        seen[path] = (data, now)
        return False
    return now-seen[path][1] >= STABLE


def needed(chapter, seen):
    """The record paths a ready chapter is staged with, or None while it is not ready."""
    root = record(chapter, 'root')
    if not stable(root, seen):
        return None
    route = json.loads(seen[root][0]).get('review_route')
    kinds = [k for k in ('independent', 'author') if (route is None and k == 'independent') or (route and k in route)]
    paths = {k: record(chapter, k) for k in kinds}
    states = [stable(p, seen) for p in paths.values()]   # every record is sighted on every poll
    if not all(states):
        return None
    return root, paths


def run(*argv):
    pinned()
    say('run: '+' '.join(str(a) for a in argv))
    subprocess.run([PYTHON, '-I', '-B', *[str(a) for a in argv]], check=True)


def main():
    assert sys.argv[1:] == [], 'no arguments'
    assert sys.executable == PYTHON and sys.flags.isolated, 'run as /usr/bin/python3 -I -B'
    assert not unresolved(), 'unresolved pins: '+', '.join(unresolved())
    pinned()
    jobs = [stage_job(ch) for ch in CHAPTERS]
    assert not any((D/'jobs'/j).exists() or (Q/(j+'-inputs.json')).exists() for j in jobs), 'a stage job exists'
    assert not MANIFEST.exists() and not (D/'jobs'/'csd-combined-master1006').exists(), 'composed manifest or master exists'
    say('waiting for the final review records of '+', '.join(CHAPTERS))
    left, seen, deadline = list(CHAPTERS), {}, time.monotonic()+LIMIT
    while left:
        for chapter in list(left):
            got = needed(chapter, seen)
            if got is None:
                continue
            root, paths = got
            extra = [a for k in ('independent', 'author') if k in paths
                     for a in ('--%s-review' % k, paths[k])]
            run(STAGE, 'stage', '--chapter', chapter, '--job', stage_job(chapter), '--root-review', root, *extra)
            left.remove(chapter)
            say('%s staged as %s' % (chapter, stage_job(chapter)))
        if left:
            assert time.monotonic() < deadline, 'no stable final review records after %d s for %s' % (LIMIT, left)
            time.sleep(POLL)
    run(STAGE, 'compose', '--stage', *jobs)
    run(MASTER, 'check', '--manifest', MANIFEST)
    run(MASTER, 'launch', '--manifest', MANIFEST, '--wait-hours', '1')
    run(VERIFY, 'check')
    run(VERIFY, 'launch-verify')
    run(VERIFY, 'launch-pictures')
    run(VERIFY, 'launch-pictures')
    say('master 1006 and verify 1007 terminal 0; final picture, motion, audio, caption and publication reviews remain open')


if __name__ == '__main__':
    main()
