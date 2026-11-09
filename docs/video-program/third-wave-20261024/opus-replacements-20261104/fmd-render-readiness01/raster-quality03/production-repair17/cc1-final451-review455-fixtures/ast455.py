"""task455 (CC1): what 451 changes relative to 377 and what 452 changes relative to 388, at AST level and as text.

For each pair: top-level function and assignment names, which are byte-equal as AST dumps, which differ, which are
added or removed, and the unified text diff of every non-equal line. For 452 vs 388 also an AST comparison after
mapping the known renames (version and file names), so any remaining difference is real logic.
Usage: python3 -I -B ast455.py Q
"""
import ast
import difflib
import hashlib
import json
from pathlib import Path
import sys

Q = Path(sys.argv[1])
PINS = {'csd-final-verify377.py': None, 'csd-final-verify451.py': 'c876caf664124e9becc1d8698608ae456978badbfca114316f93459a18186d03',
        'csd-final-controller388.py': None, 'csd-final-controller452.py': 'beb4ea0650f577d3da70731bb5a01417c2d3550b7c84eb66aa77c3335d1fceae'}
out = {'pins': {}}
for n, want in PINS.items():
    got = hashlib.sha256((Q/n).read_bytes()).hexdigest()
    assert want is None or got == want, n
    out['pins'][n] = got


def top(text):
    tree = ast.parse(text)
    items = {}
    for node in tree.body:
        if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
            items['def ' + node.name] = ast.dump(node)
        elif isinstance(node, (ast.Assign, ast.AnnAssign)):
            targets = node.targets if isinstance(node, ast.Assign) else [node.target]
            items['= ' + ','.join(ast.unparse(t) for t in targets)] = ast.dump(node)
        elif isinstance(node, (ast.Import, ast.ImportFrom)):
            items['import ' + ast.unparse(node)] = ast.dump(node)
        elif isinstance(node, ast.Expr) and isinstance(getattr(node, 'value', None), ast.Constant) and isinstance(node.value.value, str):
            items['docstring@%d' % node.lineno] = ast.dump(node)
        else:
            items['%s@%d' % (type(node).__name__, node.lineno)] = ast.dump(node)
    return items


def compare(a_name, b_name, renames=()):
    a, b = (Q/a_name).read_text(), (Q/b_name).read_text()
    ta, tb = top(a), top(b)
    if renames:
        mapped = b
        for new, old in renames:
            mapped = mapped.replace(new, old)
        tm = top(mapped)
    else:
        tm = tb
    res = dict(equal=sorted(k for k in ta if k in tb and ta[k] == tb[k]),
               differ=sorted(k for k in ta if k in tb and ta[k] != tb[k]),
               added=sorted(k for k in tb if k not in ta), removed=sorted(k for k in ta if k not in tb))
    if renames:
        res['after_renames'] = dict(differ=sorted(k for k in ta if k in tm and ta[k] != tm[k]),
                                    added=sorted(k for k in tm if k not in ta), removed=sorted(k for k in ta if k not in tm),
                                    renames=[list(r) for r in renames])
    diff = list(difflib.unified_diff(a.splitlines(), b.splitlines(), a_name, b_name, n=0, lineterm=''))
    res['diff_lines'] = len([d for d in diff if d[:1] in '+-' and d[:3] not in ('+++', '---')])
    res['diff'] = diff
    res['counts'] = dict(equal=len(res['equal']), differ=len(res['differ']), added=len(res['added']), removed=len(res['removed']))
    return res


out['451_vs_377'] = compare('csd-final-verify377.py', 'csd-final-verify451.py')
v388 = (Q/'csd-final-controller388.py').read_text()
v452 = (Q/'csd-final-controller452.py').read_text()
out['452_vs_388'] = compare('csd-final-controller388.py', 'csd-final-controller452.py')
print(json.dumps(out, indent=1))
