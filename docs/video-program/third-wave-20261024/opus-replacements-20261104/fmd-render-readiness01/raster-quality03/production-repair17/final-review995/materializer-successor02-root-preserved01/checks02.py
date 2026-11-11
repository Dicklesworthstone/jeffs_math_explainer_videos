"""checks02.py: static checks of the impl02 materializer (CC2 stage-review01 F1). Nothing in production is run or written.

1. compile() 991, impl01 and impl02; no .pyc is written;
2. unified diffs impl01 -> impl02 (the F1 change only) and 991 -> impl02, written exclusively;
3. AST: impl02 equals impl01 apart from the module docstring and the two stem-selection statements;
4. the exact stem-selection statements of impl01 and impl02, cut from their own source by AST, run on the
   score21 summary object in root's terminal record score21-terminal01.json for 01, 08, 09 and 11:
   impl01 must refuse 01 (F1 reproduced); impl02 must give 01 one whole stem, 08 and 09 three parts (09 equal
   to 991's literals) and 11 two parts, and no .parts.tsv or qa file;
5. impl02 still refuses on its unresolved pins before reading any file, and its CHAPTERS table still equals the
   unchanged gate draft's (impl01 gate 2a855393).

usage: python3 -I -B checks02.py PR17_DIR IMPL01_DIR IMPL02_DIR
"""
import ast
import difflib
import hashlib
import importlib.util
import json
import subprocess
import sys
from pathlib import Path, PurePosixPath

PR17, IMPL01, IMPL02 = (Path(x) for x in sys.argv[1:4])
FILES = {'991': PR17/'native-linux-materialize991.py', 'impl01': IMPL01/'cc0-native-materialize-repaired01.py',
         'impl02': IMPL02/'cc0-native-materialize-repaired02.py'}
TERMINAL = PR17/'score21-terminal01.json'
lines, bad = [], []
say = lines.append


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def check(name, ok):
    say('%-4s %s' % ('ok' if ok else 'FAIL', name))
    if not ok:
        bad.append(name)


for key, p in FILES.items():
    try:
        compile(p.read_text(), p.name, 'exec')
        check('compiles: %s (%s)' % (p.name, sha(p)[:8]), True)
    except SyntaxError as error:
        check('compiles: %s: %s' % (p.name, error), False)

for base in ('impl01', '991'):
    diff = list(difflib.unified_diff(FILES[base].read_text().splitlines(True), FILES['impl02'].read_text().splitlines(True),
                                     FILES[base].name, FILES['impl02'].name))
    out = IMPL02/('cc0-native-materialize-repaired02-from-%s.diff' % base)
    with open(out, 'x', encoding='utf-8') as f:
        f.writelines(diff)
    added = sum(1 for x in diff if x.startswith('+') and not x.startswith('+++'))
    removed = sum(1 for x in diff if x.startswith('-') and not x.startswith('---'))
    say('     diff %s: +%d -%d (%s)' % (out.name, added, removed, sha(out)[:8]))


def stem_block(path):
    """The statements from the stem prefix assignment through the 09 literal check, cut from the source by AST."""
    tree = ast.parse(path.read_text())
    for node in ast.walk(tree):
        for field in ('body', 'orelse'):
            body = getattr(node, field, None)
            if not isinstance(body, list):
                continue
            names = [getattr(s.targets[0], 'id', None) if isinstance(s, ast.Assign) and len(s.targets) == 1 else None
                     for s in body]
            if 'stem_names' in names:
                end = names.index('stem_names')
                block = body[end-1:end+3]       # prefix/part assignment, stem_names, assert stem_names, 09 literal check
                assert isinstance(block[2], ast.Assert) and isinstance(block[3], ast.If), path
                return tree, block
    raise AssertionError('no stem block in %s' % path)


def without(tree, block):
    """The module dump with the module docstring and the stem block removed."""
    drop = {id(s) for s in block}
    tree = ast.parse(ast.unparse(tree))  # fresh copy
    return tree


t01, b01 = stem_block(FILES['impl01'])
t02, b02 = stem_block(FILES['impl02'])
say('     impl01 stem block: ' + ' | '.join(ast.unparse(s).splitlines()[0] for s in b01[:2]))
say('     impl02 stem block: ' + ' | '.join(ast.unparse(s).splitlines()[0] for s in b02[:2]))


def strip(path):
    """Source AST with the module docstring dropped and the two selection statements replaced by a marker."""
    tree = ast.parse(path.read_text())
    tree.body = tree.body[1:] if isinstance(tree.body[0], ast.Expr) and isinstance(tree.body[0].value, ast.Constant) else tree.body
    for node in ast.walk(tree):
        for field in ('body', 'orelse'):
            body = getattr(node, field, None)
            if isinstance(body, list):
                for i, s in enumerate(body):
                    if isinstance(s, ast.Assign) and len(s.targets) == 1 and getattr(s.targets[0], 'id', None) in ('part', 'prefix', 'stem_names'):
                        body[i] = ast.Pass()
    return ast.dump(tree)


check('impl02 equals impl01 apart from the docstring and the two selection statements',
      strip(FILES['impl01']) == strip(FILES['impl02']))
check('impl02 keeps 991\'s 09 literal check unchanged from impl01', ast.dump(b01[3]) == ast.dump(b02[3]))

summary = json.loads(TERMINAL.read_text())['summary']
say('     score21 summary: root terminal record %s (%s), %d files' % (TERMINAL.name, sha(TERMINAL)[:8], len(summary['files'])))


class Master:
    SCORE = PurePosixPath('/score21')


def require(p):
    return p


def read(p):
    assert p.name == 'score21-summary.json', p
    return summary


def run(block, chapter):
    env = dict(CHAPTER=chapter, master=Master, require=require, read=read)
    exec(compile(ast.Module(body=block, type_ignores=[]), 'stem-block', 'exec'), env)
    return env['stem_names']


EXPECT = {'01_the_bump': {'01_the_bump.music.f32.wav'},
          '08_loop_sums': {'08_loop_sums.music.f32.wav.part%04d.wav' % i for i in range(3)},
          '09_boundary': {'09_boundary.music.f32.wav.part%04d.wav' % i for i in range(3)},
          '11_counting': {'11_counting.music.f32.wav.part%04d.wav' % i for i in range(2)}}
for chapter, want in EXPECT.items():
    listed = sorted(k for k in summary['files'] if k.startswith('stems/'+chapter+'.') or k.startswith('stems/qa/'+chapter+'.'))
    say('     score21 %s: %s' % (chapter, listed))
    try:
        got01 = run(b01, chapter)
        say('     impl01 %s -> %s' % (chapter, sorted(got01)))
        check('impl01 %s %s' % (chapter, 'refuses (F1)' if chapter == '01_the_bump' else 'unchanged'),
              chapter != '01_the_bump' and got01 == want)
    except AssertionError:
        check('impl01 %s refuses (F1 reproduced)' % chapter, chapter == '01_the_bump')
    got = run(b02, chapter)
    say('     impl02 %s -> %s' % (chapter, sorted(got)))
    check('impl02 %s gives exactly the expected stems' % chapter, got == want)
    check('impl02 %s takes no parts table or qa file' % chapter, all(x.endswith('.wav') and '.parts.' not in x for x in got))

run_ = subprocess.run([sys.executable, '-I', '-B', str(FILES['impl02']), '--chapter', '01_the_bump', '--job', 'none',
                       '--inputs', 'none', '--proofs', 'none', '--manifest', 'none'], capture_output=True, text=True, timeout=60)
last = run_.stderr.strip().splitlines()[-1] if run_.stderr.strip() else ''
say('     impl02 with placeholders: exit %d, %s' % (run_.returncode, last[:150]))
check('impl02 refuses on unresolved pins', run_.returncode != 0 and 'unresolved pins' in last)


def load(p, name):
    spec = importlib.util.spec_from_file_location(name, p)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


m, g = load(FILES['impl02'], 'materialize02'), load(IMPL01/'cc0-native-master-gate-repaired01.py', 'gate01')
check('gate draft unchanged (2a855393)', sha(IMPL01/'cc0-native-master-gate-repaired01.py').startswith('2a855393'))
for key in m.CHAPTERS:
    for field in ('kind', 'build', 'binary', 'tests', 'delivery', 'timeline', 'preview', 'pictures',
                  'frames', 'scene', 'source_sha', 'staged993'):
        if m.CHAPTERS[key][field] != g.CHAPTERS[key][field]:
            check('%s %s equal in impl02 and the gate draft' % (key, field), False)
check('CHAPTERS fields equal in impl02 and the gate draft', not any('equal in impl02' in b for b in bad))
check('impl02 lists 5 pins', len(m.unresolved()) == 5)
say('RESULT: %s' % ('all checks passed' if not bad else 'FAILED: '+'; '.join(bad)))
with open(IMPL02/'checks02.out', 'x', encoding='utf-8') as f:
    f.write('\n'.join(lines)+'\n')
print('\n'.join(lines))
