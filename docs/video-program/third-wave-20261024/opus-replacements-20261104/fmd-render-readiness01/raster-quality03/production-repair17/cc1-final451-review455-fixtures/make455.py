"""task455 (CC1): derives the 452 harness from the 391 harness (which ran 388 against a 377 stand-in) by explicit
replacements only, each counted; writes new files, x-mode. The 391 files are read, never changed.
  fake451_gen455.py  <- fake377_gen391c.py   (builds the 451 stand-in from 451's own source)
  run452_455.py      <- run388_391.py        (executes the frozen 452 unchanged, rebound to a scratch D)
  test455.py         <- test391.py           (the same 26 scenarios, 451/452 names)
  fakeguard455.py    <- fakeguard391.py      (root's guard runner 819746fb, unchanged code)
Usage: python3 -I -B make455.py DIR391 DIR455
"""
import hashlib
import json
from pathlib import Path
import sys

A, B = Path(sys.argv[1]), Path(sys.argv[2])
H377, H451 = '827be351861d0fd4ebc083aa24d594111b059dd9f78c7ca43072b0b6f12f22fe', 'c876caf664124e9becc1d8698608ae456978badbfca114316f93459a18186d03'
H388, H452 = '5599e007e6b1debf37fe98a22ad1db322896e446a07d3f837fcddcd9e33fb59e', 'beb4ea0650f577d3da70731bb5a01417c2d3550b7c84eb66aa77c3335d1fceae'
GENERIC = [('cc0-combined-master350.py', 'combined-master447.py'), ('377', '451'), ('388', '452'), ('391', '455'), ('350', '447')]
PLAN = {'fake451_gen455.py': ('fake377_gen391c.py', [(H377, H451)]),
        'run452_455.py': ('run388_391.py', [(H388, H452)]),
        'test455.py': ('test391.py', []),
        'fakeguard455.py': ('fakeguard391.py', [])}
report = {}
for new, (old, first) in PLAN.items():
    text = (A/old).read_text()
    counts = []
    for a, b in first + GENERIC:
        n = text.count(a)
        counts.append([a[:16], b[:16], n])
        text = text.replace(a, b)
    if first:
        assert counts[0][2] == 1, (new, counts[0])
    compile(text, new, 'exec')
    with open(B/new, 'x') as f:
        f.write(text)
    report[new] = dict(source=old, source_sha256=hashlib.sha256((A/old).read_bytes()).hexdigest(),
                       sha256=hashlib.sha256(text.encode()).hexdigest(), replacements=counts)
print(json.dumps(report, indent=1))
