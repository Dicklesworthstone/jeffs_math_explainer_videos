"""task391 static checks of 388 against 377 (read-only AST). Usage: python3 -I -B static391.py C388 V377 SHARED"""
import ast
import hashlib
import json
import re
import sys

c_src, v_src, s_src = (open(p).read() for p in sys.argv[1:4])
for p, want in zip(sys.argv[1:4], ('5599e007', '827be351', 'f266a231')):
    assert hashlib.sha256(open(p, 'rb').read()).hexdigest().startswith(want), p
c, v, s = ast.parse(c_src), ast.parse(v_src), ast.parse(s_src)
out = {}
# 1. top-level statements: no I/O at load
for name, tree in (('388', c), ('377', v), ('shared', s)):
    kinds = sorted({type(n).__name__ for n in tree.body})
    calls = sorted({ast.unparse(n.func) for n in ast.walk(ast.Module(body=[x for x in tree.body if not isinstance(x, (ast.FunctionDef, ast.ClassDef, ast.If))], type_ignores=[])) if isinstance(n, ast.Call)})
    out[name + '_toplevel'] = dict(kinds=kinds, calls=calls)
# 2. every write-capable call in 388
writes = []
for n in ast.walk(c):
    if isinstance(n, ast.Call):
        f = ast.unparse(n.func)
        if re.search(r'(open|write|mkdir|unlink|rename|replace|remove|rmtree|touch|symlink|chmod)$', f):
            writes.append(dict(line=n.lineno, call=ast.unparse(n)[:90]))
out['388_write_capable_calls'] = writes
# 3. subprocess use in 388
out['388_subprocess'] = [dict(line=n.lineno, call=ast.unparse(n)[:120]) for n in ast.walk(c)
                         if isinstance(n, ast.Call) and ast.unparse(n.func).startswith('subprocess.')]
out['388_step_modes'] = [ast.unparse(n)[:100] for n in ast.walk(c) if isinstance(n, ast.Call) and ast.unparse(n.func) == 'step']
# 4. names 388 reads from v exist at 377's top level
used = sorted({n.attr for n in ast.walk(c) if isinstance(n, ast.Attribute) and isinstance(n.value, ast.Name) and n.value.id == 'v'})
top = {t.id for n in v.body if isinstance(n, ast.Assign) for t in n.targets if isinstance(t, ast.Name)} | \
      {n.name for n in v.body if isinstance(n, ast.FunctionDef)}
out['v_names_used'] = used
out['v_names_missing'] = [x for x in used if x not in top]
# 5. the last-line regex against 377's own format string
fmt = None
for n in ast.walk(v):
    if isinstance(n, ast.BinOp) and isinstance(n.op, ast.Mod) and isinstance(n.left, ast.Constant) and \
            str(n.left.value).startswith('%d pictures in %d sheets'):
        fmt = n.left.value
rx = None
for n in ast.walk(c):
    if isinstance(n, ast.Assign) and getattr(n.targets[0], 'id', None) == 'LAST_LINE':
        rx = re.compile(ast.literal_eval(n.value.args[0]))
cases = {'rendered': fmt % (19, 3, 'a' * 64), 'zero_sheets': fmt % (0, 0, '0' * 64),
         'upper_hex': fmt % (19, 3, 'A' * 64), 'short_hash': fmt % (19, 3, 'a' * 63),
         'trailing_space': fmt % (19, 3, 'a' * 64) + ' ', 'native_line': "3 native 4K frames await root's viewing; "}
out['last_line_regex'] = {k: bool(rx.match(x)) for k, x in cases.items()}
# 6. keys 388 parses vs keys 377 prints (launch: job + **note; check: check/frames/picture_jobs/plan_sha256)
prints = {}
for fn in v.body:
    if isinstance(fn, ast.FunctionDef) and fn.name in ('launch', 'main'):
        for n in ast.walk(fn):
            if isinstance(n, ast.Call) and ast.unparse(n.func) == 'json.dumps' and isinstance(n.args[0], ast.Call) \
                    and ast.unparse(n.args[0].func) == 'dict':
                prints[fn.name] = sorted(k.arg if k.arg else '**' + ast.unparse(k.value) for k in n.args[0].keywords)
out['377_printed_dict_keys'] = prints
note_calls = [ast.unparse(n)[:160] for fn in v.body if isinstance(fn, ast.FunctionDef) and fn.name == 'main'
              for n in ast.walk(fn) if isinstance(n, ast.Call) and ast.unparse(n.func) == 'launch']
out['377_launch_calls'] = note_calls
print(json.dumps(out, indent=1))
