"""task455 trial b (CC1): the two residuals ast455.py left open.
1. 451 vs 377: the main block (If@1218 in 377, If@1245 in 451) and the old docstring (377 line 1, 451 line 4) are
   AST-equal; every other top-level node is accounted for.
2. 452 vs 388: after mapping 452's names back to 388's (hash, ledger, then 452->388, 451->377, 447->350, and the
   docstring's 'csd-combined-master447 (cc0-combined-master447.py' wording), the whole module AST equals 388's.
Usage: python3 -I -B ast455b.py Q
"""
import ast
import hashlib
import json
from pathlib import Path
import sys

Q = Path(sys.argv[1])
out = {}
t377, t451 = (ast.parse((Q/n).read_text()) for n in ('csd-final-verify377.py', 'csd-final-verify451.py'))
main377 = [n for n in t377.body if isinstance(n, ast.If)]
main451 = [n for n in t451.body if isinstance(n, ast.If)]
out['main_blocks'] = dict(n377=len(main377), n451=len(main451), equal=[ast.dump(a) == ast.dump(b) for a, b in zip(main377, main451)],
                          text=ast.unparse(main451[0]))
doc377 = t377.body[0]
doc451 = [n for n in t451.body[:2]]
out['docstrings'] = dict(old_docstring_kept_as_second_expression=ast.dump(doc451[1]) == ast.dump(doc377),
                         new_first_docstring=doc451[0].value.value)
# 451 body = 377 body with: the new docstring first, 4 new constants and 2 new functions after WORKERS-related code,
# and changed nodes; check by sequence alignment of dumps
import difflib
d377 = [ast.dump(n) for n in t377.body]
d451 = [ast.dump(n) for n in t451.body]
ops = []
for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(a=d377, b=d451, autojunk=False).get_opcodes():
    if tag != 'equal':
        ops.append(dict(tag=tag, n377=[ast.unparse(t377.body[i]).splitlines()[0][:90] for i in range(i1, i2)],
                        n451=[ast.unparse(t451.body[j]).splitlines()[0][:90] for j in range(j1, j2)]))
out['451_body_ops'] = ops
out['451_equal_nodes'] = sum(i2 - i1 for tag, i1, i2, j1, j2 in difflib.SequenceMatcher(a=d377, b=d451, autojunk=False).get_opcodes() if tag == 'equal')
out['n377_nodes'], out['n451_nodes'] = len(d377), len(d451)

t388 = (Q/'csd-final-controller388.py').read_text()
t452 = (Q/'csd-final-controller452.py').read_text()
renames = [('c876caf664124e9becc1d8698608ae456978badbfca114316f93459a18186d03', '827be351861d0fd4ebc083aa24d594111b059dd9f78c7ca43072b0b6f12f22fe'),
           ('final-pictures451-overview-ledger452', 'final-pictures377-overview-ledger388'),
           ('cc0-combined-master447.py', 'cc0-combined-master350.py'),
           ('452', '388'), ('451', '377'), ('447', '350')]
mapped = t452
counts = []
for a, b in renames:
    counts.append([a[:20], b[:20], mapped.count(a)])
    mapped = mapped.replace(a, b)
a388, a452m = ast.parse(t388), ast.parse(mapped)
out['452_mapped'] = dict(renames=counts, module_ast_equal_388=ast.dump(a388) == ast.dump(a452m),
                         text_equal_388=mapped == t388,
                         remaining_text_diff=[l for l in difflib.unified_diff(t388.splitlines(), mapped.splitlines(), n=0, lineterm='')
                                              if l[:1] in '+-' and l[:3] not in ('+++', '---')])
print(json.dumps(out, indent=1))
