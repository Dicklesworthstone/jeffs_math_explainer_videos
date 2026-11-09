"""task455 (CC1): the 26 scenario outcomes of 452 (test455) against those of 388 (test391), after renames.
Compares exit, result text (scratch paths and names normalized), the modes run, printed JSON key sets, the ledger
line, every job's receipt state and the ledger/sampling/native bindings. Timestamps and hashes are dropped.
Usage: python3 -I -B compare455.py test391.out.json test455.out.json
"""
import json
import re
import sys

a, b = (json.load(open(p)) for p in sys.argv[1:3])
RN = [('377', 'V'), ('451', 'V'), ('388', 'C'), ('452', 'C'), ('350', 'M'), ('447', 'M'), ('391', 'R'), ('455', 'R')]


def norm(x):
    if isinstance(x, dict):
        return {norm(k): norm(v) for k, v in x.items() if k not in ('started', 'finished_utc', 'ledger_written_utc', 'ledger_sha256',
                                                                     'ledger_overview_sampling_sha256')}
    if isinstance(x, list):
        return [norm(v) for v in x]
    if isinstance(x, str):
        x = re.sub(r'\S*/task(391|455)/runs/', 'RUNS/', x)
        x = re.sub(r'[0-9a-f]{64}', 'HASH', x)
        for s, t in RN:
            x = x.replace(s, t)
        return x
    return x


sa, sb = norm(a['scenarios']), norm(b['scenarios'])
diff = {}
for k in sorted(set(sa) | set(sb)):
    if sa.get(k) != sb.get(k):
        x, y = sa.get(k) or {}, sb.get(k) or {}
        diff[k] = {f: [x.get(f), y.get(f)] for f in sorted(set(x) | set(y)) if x.get(f) != y.get(f)}
print(json.dumps(dict(scenarios_391=len(sa), scenarios_455=len(sb), identical=sorted(k for k in sa if sa.get(k) == sb.get(k)),
                      differ=diff, inputs_455={k: b[k] for k in ('real388', 'runner', 'fake377', 'fakeguard', 'run388', 'python')
                                               if k in b} or {k: v for k, v in b.items() if k != 'scenarios'}), indent=1))
