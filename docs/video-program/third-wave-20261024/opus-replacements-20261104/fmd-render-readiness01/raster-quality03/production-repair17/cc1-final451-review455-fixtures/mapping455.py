"""task455 (CC1): positive and negative row mapping, master447 -> verifier451, on the actual draft449 rows.

Loads 447 and 451 unchanged (hash-checked; their top levels do no I/O, checked by AST below). For each of the 15
draft449 rows it derives what 447's chapter_gate records (host by 447's rule, and the source job 447 reads, from
447's own combined_source447), then applies 451's expected_worker451 and source_job451 exactly as 451's master350()
does, and requires 451's source == 447's source for all 15 rows. Then mutated rows that 451 must refuse.
No filesystem access beyond reading the three source files.
Usage: python3 -I -B mapping455.py Q
"""
import ast
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

Q = Path(sys.argv[1])
PINS = {'combined-master447.py': '2b01ecaab2b409268d86fe13b83e97a7e374dd61f1c657680a32444fdb6aa524',
        'csd-final-verify451.py': 'c876caf664124e9becc1d8698608ae456978badbfca114316f93459a18186d03',
        'master447-manifest-draft449.json': None}
for name, want in PINS.items():
    got = hashlib.sha256((Q/name).read_bytes()).hexdigest()
    PINS[name] = got if want is None else want
    assert got == PINS[name], name
SAFE = {'Path', 're.compile', 'dict', 'set', 'tuple', 'frozenset', 'range', 'list'}
out = {'pins': PINS, 'toplevel_calls': {}}


def load(name):
    text = (Q/name).read_text()
    tree = ast.parse(text)
    body = [n for n in tree.body if not isinstance(n, (ast.FunctionDef, ast.ClassDef, ast.If))]
    calls = sorted({ast.unparse(n.func) for n in ast.walk(ast.Module(body=body, type_ignores=[])) if isinstance(n, ast.Call)})
    out['toplevel_calls'][name] = calls
    assert set(calls) <= SAFE | {'Path(__file__).resolve'}, (name, calls)
    spec = importlib.util.spec_from_file_location(name.replace('-', '_')[:-3], Q/name)
    m = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(m)
    return m


m447, v451 = load('combined-master447.py'), load('csd-final-verify451.py')
rows = json.loads((Q/'master447-manifest-draft449.json').read_text())['chapters']
assert [r['chapter'] for r in rows] == sorted(v451.KINDS)


def record447(row):
    """The host, worker, worker_sha256 and job 447's chapter_gate writes into its record, and the source job it
    reads: 447 :926-947, :976, :1105-1106."""
    key, kind = row['chapter'], row['kind']
    worker, job = Path(row['worker']), row['job']
    if kind == 'retained':
        host = m447.RETAINED[key]['host']
        source = Path(host)/'jobs'/job
    elif kind == 'boundary313':
        host, source = m447.D, m447.D/'jobs'/job
    else:
        assert worker in m447.WORKERS352 and m447.WORKERS352[worker] == row['worker_sha256']
        host = m447.D if worker == m447.WORKER352 else m447.CSS
        source = m447.combined_source447(worker, key, job)
    return dict(chapter=key, kind=kind, host=str(host), worker=str(worker), worker_sha256=row['worker_sha256'], job=job), source


def check451(key, rec):
    """451 master350() :441-450 and :481 on one record; returns 451's source job."""
    kind = v451.KINDS[key]
    assert rec['kind'] == kind
    if kind != 'retained':
        worker, digest, job = v451.expected_worker451(key, rec)
        assert (rec['worker'], rec['worker_sha256'], rec['job']) == \
            (str(worker), digest, job % key.replace('_', '-') if '%s' in job else job), key+': not the frozen 344/352 worker and job'
    return v451.source_job451(key, rec)


positive = []
for row in rows:
    rec, source447 = record447(row)
    source451 = check451(row['chapter'], rec)
    positive.append(dict(chapter=row['chapter'], kind=row['kind'], host=rec['host'], source447=str(source447),
                         source451=str(source451), equal=source447 == source451))
out['positive'] = positive
out['positive_all_equal'] = all(p['equal'] for p in positive) and len(positive) == 15
by = {r['chapter']: r for r in rows}
g = dict(record447(by['05_gallery'])[0])
c04 = dict(record447(by['04_reading_color'])[0])
d06 = dict(record447(by['06_one_derivative'])[0])
negatives = {
    'gallery host recorded as D': ('05_gallery', dict(g, host=str(v451.D))),
    'gallery on the old CSS 352 worker and failed job': ('05_gallery', dict(g, worker=str(v451.WORKERS[('combined', v451.CSS)][0]),
        worker_sha256=v451.WORKERS[('combined', v451.CSS)][1], job='css-reviewed-final-05-gallery352')),
    'gallery on the CSD 352 worker': ('05_gallery', dict(g, host=str(v451.D), worker=str(v451.WORKERS[('combined', v451.D)][0]),
        worker_sha256=v451.WORKERS[('combined', v451.D)][1], job='csd-reviewed-final-05-gallery352')),
    'gallery job of failed attempt 440': ('05_gallery', dict(g, job='css-reviewed-final-05-gallery352-ssd440')),
    'gallery worker sha of 440': ('05_gallery', dict(g, worker_sha256=hashlib.sha256((Q/'css-gallery-ssd440.py').read_bytes()).hexdigest())),
    'gallery kind retained': ('05_gallery', dict(g, kind='retained')),
    'SSD442 worker on 04': ('04_reading_color', dict(c04, worker=g['worker'], worker_sha256=g['worker_sha256'])),
    'SSD442 worker on 06 (CSD host)': ('06_one_derivative', dict(d06, worker=g['worker'], worker_sha256=g['worker_sha256'])),
    'SSD442 job name on 04': ('04_reading_color', dict(c04, job='css-reviewed-final-05-gallery352-ssd442')),
}
refused = {}
for name, (key, rec) in negatives.items():
    try:
        check451(key, rec)
        refused[name] = 'ACCEPTED'
    except (AssertionError, KeyError) as error:
        refused[name] = '%s: %s' % (type(error).__name__, str(error)[:120])
out['negatives'] = refused
out['negatives_all_refused'] = all(v != 'ACCEPTED' for v in refused.values())
# 447 itself on the same mutations (does 447 let them through to a master?)
m447_view = {}
for name, (key, rec) in negatives.items():
    try:
        w = Path(rec['worker'])
        if rec['kind'] == 'combined':
            assert w in m447.WORKERS352 and m447.WORKERS352[w] == rec['worker_sha256']
            m447.combined_source447(w, key, rec['job'])
        else:
            m447.RETAINED[key]
        m447_view[name] = 'passes 447 row mapping'
    except (AssertionError, KeyError) as error:
        m447_view[name] = '447 refuses: %s' % type(error).__name__
out['same_mutations_at_447_row_mapping'] = m447_view
print(json.dumps(out, indent=1))
