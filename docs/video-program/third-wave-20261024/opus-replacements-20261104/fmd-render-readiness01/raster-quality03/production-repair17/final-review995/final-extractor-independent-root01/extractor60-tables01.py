"""Read-only: test extractor 60addbf3's post-decode table checks (normalized(), the 09 replace) on actual bytes,
reconstructed from local reference tables and verified against the combined01 comparison's actual_sha256.
normalized() is cut by AST from CC0's frozen file (no PIL import, main() not run).
usage: python3 -I -B extractor60-tables01.py EXTRACTOR COMPARISON_JSON CANDIDATE_TSV..."""
import ast
import hashlib
import json
import sys
from pathlib import Path

src = Path(sys.argv[1]).read_text()
assert hashlib.sha256(src.encode()).hexdigest().startswith('60addbf3')
tree = ast.parse(src)
ns = {'hashlib': hashlib, 'R': Path('/data/projects/jmev-stroke-probe-20261106')}
for node in tree.body:
    if isinstance(node, ast.FunctionDef) and node.name == 'normalized':
        exec(compile(ast.Module([node], []), 'normalized', 'exec'), ns)
    if isinstance(node, ast.Assign) and getattr(node.targets[0], 'id', '') in ('NORMALIZE_TO', 'OLD09', 'EXPECTED09'):
        exec(compile(ast.Module([node], []), 'const', 'exec'), ns)
normalized = ns['normalized']
comp = json.loads(Path(sys.argv[2]).read_text())['chapters']
R = '/data/projects/jmev-stroke-probe-20261106'
NARRATION = R + '/jobs/csd-native-combined-delivery01/narration'
SPEC = {'01_the_bump': '150147fe5bc7cdb0a111061c4530a16b104d196e004ae29bcf210d1c9c32ac81',
        '08_loop_sums': '43c8bab2f7ca61c8692e0f1686798d54e187dcfeeab0d03311c2d4c9a5f54812',
        '09_boundary': '5a91b8f9e2e5b5d6223a49285c06039b1b43bd16c11cecdd163a8e2e545baa9e'}
for p in map(Path, sys.argv[3:]):
    chapter = p.name.split('.')[0]
    text = p.read_text()
    folders = {line.split('\t')[-1].rsplit('/', 1)[0] for line in text.split('\n') if line.split('\t')[0] == 'line'}
    if len(folders) != 1:
        print(p, 'skip: line-row folders', folders)
        continue
    old = folders.pop()
    actual = text.replace(old + '/', NARRATION + '/')
    a_sha = hashlib.sha256(actual.encode()).hexdigest()
    ok_actual = a_sha == comp[chapter]['actual_sha256']
    line = f'{chapter} from {p.parent.name}/{p.name} (folder {old.rsplit("/", 2)[-2]}): rebuilt actual {a_sha[:8]} ' \
           f'{"== comparison actual" if ok_actual else "!= comparison actual " + comp[chapter]["actual_sha256"][:8]}'
    if ok_actual:
        n = normalized(actual, NARRATION, ns['NORMALIZE_TO'][chapter])
        line += f'; normalized() {n[:8]} {"== runner spec" if n == SPEC[chapter] else "!= runner spec " + SPEC[chapter][:8]}'
        if chapter == '09_boundary':
            r = hashlib.sha256(actual.encode().replace(NARRATION.encode(), ns['OLD09'].encode())).hexdigest()
            line += f'; 09 replace {r[:8]} {"== EXPECTED09" if r == ns["EXPECTED09"]["timeline.tsv"] else "!= EXPECTED09"}'
    print(line)
