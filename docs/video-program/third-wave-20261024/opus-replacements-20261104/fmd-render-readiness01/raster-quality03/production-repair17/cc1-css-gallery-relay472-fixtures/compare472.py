"""relay472 (CC1): every difference between review 466's 463 outputs (FIX466) and the 472 reruns (run472 OUTDIR).
Normalized before comparing: ISO times -> TS, hex runs of 12+ -> HEX, scratch path fragments (raw or cleaned) -> PATH,
a-h condition labels -> their letter. Prints each differing leaf as path: 463 -> 472.
Usage: python3 -I -B compare472.py FIX466 OUTDIR"""
import json
import re
import sys
from pathlib import Path

FIX, OUT = Path(sys.argv[1]), Path(sys.argv[2])
PAIRS = (('test466.out.json', 'runs-a/test466.out.json'), ('test466b.out.json', 'runs-b/test466b.out.json'),
         ('test466c.out.json', 'runs-c/test466c.out.json'))
TRUNC = re.compile(r'W/\[path truncated before cleaning[^\]]*\]|/private/tmp/claude-501\S*')


def norm(x):
    if isinstance(x, dict):
        return {k: norm(v) for k, v in x.items()}
    if isinstance(x, list):
        return [norm(v) for v in x]
    if isinstance(x, str):
        m = re.match(r'^([a-h])\. ', x)
        if m:
            return m.group(1)
        x = TRUNC.sub('PATH', x)
        x = re.sub(r'\d{4}-\d\d-\d\dT[\d:.]+\+00:00', 'TS', x)
        return re.sub(r'\b[0-9a-f]{12,64}\b', 'HEX', x)
    return x


def walk(a, b, path, out):
    if isinstance(a, dict) and isinstance(b, dict):
        for k in sorted(set(a) | set(b)):
            walk(a.get(k, '<absent>'), b.get(k, '<absent>'), path+'/'+k, out)
    elif a != b:
        out.append((path, a, b))


diffs = []
for old, new in PAIRS:
    walk(norm(json.loads((FIX/old).read_text())), norm(json.loads((OUT/new).read_text())), old, diffs)
for path, a, b in diffs:
    print('%s: %s -> %s' % (path, json.dumps(a)[:220], json.dumps(b)[:220]))
print('differences:', len(diffs))
