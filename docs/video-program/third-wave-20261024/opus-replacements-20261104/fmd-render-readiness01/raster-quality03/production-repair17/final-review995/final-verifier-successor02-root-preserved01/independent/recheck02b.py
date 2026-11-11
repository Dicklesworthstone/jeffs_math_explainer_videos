"""recheck02b.py: CC0's follow-up to recheck02 on CC3's successor02 draft (1220fdb1). recheck02's three FAILs are its
own text checks (tokens equal to ALT; nothing removed beyond fix03's return line; no line beyond the snapshot,
condition and comment), and all three name the module docstring, which CC3 said changes. This checks exactly that:

1. successor02's AST equals ALT 0f31010e's once the module docstring is removed (independent of CC3's own check);
2. its Python tokens equal ALT's apart from comments, non-logical line breaks and the module docstring token;
3. the module docstring differs from ALT's (= 5c30a62e's + the fix bullet) only in its first line, which becomes two;
4. no line successor02 adds to 5c30a62e carries a local path or a flagged word.
Read-only for every input; writes only its own output.

usage: python3 -I -B recheck02b.py CC3_BASE_FILE ALT0_FILE SUCCESSOR02_FILE OUT_FILE
"""
import ast
import difflib
import hashlib
import io
import re
import sys
import tokenize
from pathlib import Path

BASE, ALT0, NEW, OUT = (Path(x) for x in sys.argv[1:5])
lines, bad = [], []
say = lines.append


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def check(name, ok):
    say('%-4s %s' % ('ok' if ok else 'FAIL', name))
    if not ok:
        bad.append(name)


assert sha(BASE) == '5c30a62ef9341ff72be6650d462f7de2eb32ad0c5089b826d50897d3c9ef4dbb'
assert sha(ALT0) == '0f31010e5f328490450e13d7b23a76ce3ba28d87fc979354dbfe3da323331f3c'
say('     successor02 %s %s' % (NEW.name, sha(NEW)))


def body(p):
    tree = ast.parse(Path(p).read_text())
    doc = ast.get_docstring(tree, clean=False)
    first = tree.body[0]
    assert isinstance(first, ast.Expr) and isinstance(first.value, ast.Constant) and first.value.value == doc
    tree.body = tree.body[1:]
    return ast.dump(tree), doc


a_body, a_doc = body(ALT0)
n_body, n_doc = body(NEW)
b_body, b_doc = body(BASE)
check('1. successor02 AST equals ALT 0f31010e without the module docstring', a_body == n_body)


def code_tokens(p):
    toks = [t for t in tokenize.generate_tokens(io.StringIO(Path(p).read_text()).readline)
            if t.type not in (tokenize.COMMENT, tokenize.NL)]
    assert toks[0].type == tokenize.STRING
    return [(t.type, t.string) for t in toks[1:]]


check('2. successor02 tokens equal ALT\'s apart from comments, line breaks and the module docstring',
      code_tokens(ALT0) == code_tokens(NEW))
d = [x for x in difflib.unified_diff(a_doc.splitlines(), n_doc.splitlines(), lineterm='', n=0)
     if not x.startswith(('---', '+++', '@@'))]
for x in d:
    say('       docstring %s' % x)
a_lines, n_lines = a_doc.splitlines(), n_doc.splitlines()
check('3. the module docstring differs from ALT\'s only in its first line, which becomes two',
      a_lines[1:] == n_lines[2:] and a_lines[0] != n_lines[0])
check('   ALT\'s docstring is 5c30a62e\'s plus the fix bullet (two lines)',
      [x for x in difflib.ndiff(b_doc.splitlines(), a_doc.splitlines()) if x.startswith(('+ ', '- '))] ==
      ['+ ' + x for x in a_lines if 'master993\'s sampled intervals of 01/08/09/11' in x or
       'them, as stills of the new final only; M1 keeps' in x])
added = [x[1:] for x in difflib.unified_diff(BASE.read_text().splitlines(), NEW.read_text().splitlines(), lineterm='')
         if x.startswith('+') and not x.startswith('+++')]
flag = re.compile(r'/Users/|/home/|/private/tmp|/private/var|jemanuel|mortimer|gracieandjeff|scratchpad|\blisten|\bwatch')
hits = [x for x in added if flag.search(x)]
for x in hits:
    say('       flagged added line: %s' % x)
check('4. no line successor02 adds to 5c30a62e carries a local path or a flagged word (%d added lines)' % len(added), not hits)
say('RESULT: %s' % ('all checks passed' if not bad else 'FAILED: ' + '; '.join(bad)))
with open(OUT, 'x', encoding='utf-8') as stream:
    stream.write('\n'.join(lines) + '\n')
print('\n'.join(lines))
