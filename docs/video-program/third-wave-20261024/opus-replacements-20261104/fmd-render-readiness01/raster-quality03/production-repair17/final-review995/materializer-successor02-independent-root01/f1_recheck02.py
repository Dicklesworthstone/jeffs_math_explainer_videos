"""F1 recheck: run the stem-selection statements cut by AST from each materializer draft against the actual
CSD score21-summary.json copy, then replay impl02's cuelog stem loop for 01 on a real scored 01 cuelog
(final01-terminal142). Local only; reads files, writes nothing. Usage: impl01 impl02 summary cuelog01"""
import ast, json, sys
from pathlib import Path

impl01, impl02, summary_path, cuelog01 = map(Path, sys.argv[1:5])
SCORE = Path('/data/projects/jmev-stroke-probe-20261106/jobs/csd-production-score21')
STEMS = SCORE/'score/stems'
summary = json.loads(summary_path.read_text())

def selection(path):
    """The statements from the one that names the stem prefix (or 'part') through 'assert stem_names'."""
    src = path.read_text()
    main = next(n for n in ast.walk(ast.parse(src)) if isinstance(n, ast.FunctionDef) and n.name == 'main')
    body = main.body
    start = next(i for i, n in enumerate(body) if isinstance(n, ast.Assign) and isinstance(n.targets[0], ast.Name) and n.targets[0].id in ('part', 'prefix'))
    end = next(i for i, n in enumerate(body) if isinstance(n, ast.If) and 'stem_names' in ast.unparse(n))
    return '\n'.join(ast.get_source_segment(src, n) for n in body[start:end+1])

class Master: pass
master = Master(); master.SCORE = SCORE
out = {}
for name, path in (('impl01', impl01), ('impl02', impl02)):
    code = selection(path)
    out[name+'_code'] = code
    for ch in ('01_the_bump', '08_loop_sums', '09_boundary', '11_counting'):
        env = dict(CHAPTER=ch, master=master, read=lambda p: summary, require=lambda p: p)
        try:
            exec(code, env)
            out[name+' '+ch] = sorted(env['stem_names'])
        except AssertionError as e:
            out[name+' '+ch] = 'REFUSED '+repr(e)

# impl02's cuelog stem loop on the real scored 01 cuelog: stem rows must be STEMS/<name in stem_names> at the
# summary digest, and every name used exactly once
env = dict(CHAPTER='01_the_bump', master=master, read=lambda p: summary, require=lambda p: p)
exec(selection(impl02), env)
names, stems = env['stem_names'], []
for line in cuelog01.read_text().splitlines():
    if not line.strip() or line.startswith('#'): continue
    fields = line.split('\t'); assert len(fields) == 13
    src = Path(fields[0])
    if src.parent == STEMS:
        assert src.name in names, src
        assert summary['files']['stems/'+src.name] == fields[1], src
        stems.append(src.name)
out['impl02 01 cuelog stem rows'] = stems
out['impl02 01 set(stems)==stem_names and counts equal'] = set(stems) == names and len(stems) == len(names)
print(json.dumps(out, indent=1))
