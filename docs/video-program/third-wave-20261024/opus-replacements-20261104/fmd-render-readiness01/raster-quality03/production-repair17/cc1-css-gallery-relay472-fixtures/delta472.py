"""relay472 (CC1): top-level statement delta 463 -> 472 (ast, by name; the module docstring reported separately).
Usage: python3 -I -B delta472.py Q"""
import ast
import hashlib
import json
import sys
from pathlib import Path

Q = Path(sys.argv[1])
FILES = {'463': ('cc3-css-gallery-relay463.py', 'd1822a5157895ae19c7128a1de374dbafa7de204aeba8c99b3895fabe4d5b892'),
         '472': ('cc1-css-gallery-relay472.py', None)}


def statements(text):
    tree = ast.parse(text)
    out, doc = {}, ast.get_docstring(tree)
    for node in tree.body[1:]:
        if isinstance(node, (ast.FunctionDef, ast.ClassDef)):
            name = node.name
        elif isinstance(node, ast.Assign):
            name = ','.join(t.id for t in node.targets if isinstance(t, ast.Name))
        else:
            name = ast.dump(node)[:60]
        assert name not in out, name
        out[name] = ast.get_source_segment(text, node)
    return doc, out


texts = {}
for k, (name, want) in FILES.items():
    data = (Q/name).read_bytes()
    if want:
        assert hashlib.sha256(data).hexdigest() == want
    texts[k] = data.decode()
(d0, s0), (d1, s1) = statements(texts['463']), statements(texts['472'])
print(json.dumps(dict(sha472=hashlib.sha256(texts['472'].encode()).hexdigest(), lines=(texts['463'].count('\n'), texts['472'].count('\n')),
                      docstring_changed=d0 != d1, order_kept=[n for n in s1 if n in s0] == [n for n in s0 if n in s1],
                      removed=sorted(set(s0) - set(s1)), added=[n for n in s1 if n not in s0],
                      changed=[n for n in s0 if n in s1 and s0[n] != s1[n]],
                      unchanged=len([n for n in s0 if n in s1 and s0[n] == s1[n]])), indent=1))
