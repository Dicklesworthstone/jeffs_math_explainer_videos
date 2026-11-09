"""relay472 (CC1): review 466's trials a, b and c, rerun unchanged against relay 472 in place of 463.
sim466.load463 is replaced by sim472.load472 (same rebinding; 472's bytes at the hash given). Each test file is
checked against MANIFEST466. Trial a gets one textual substitution, its record-folder glob 'relay463-...' ->
'relay472-...'; trials b and c run byte for byte. Fixture only; no actual import.
Usage: python3 -I -B run472.py Q FIX466 H472 OUTDIR   (OUTDIR new; runs-a, runs-b, runs-c go in it)"""
import contextlib
import hashlib
import json
import sys
from pathlib import Path

Q0, FIX, H472, OUT = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve(), sys.argv[3], Path(sys.argv[4]).resolve()
assert not OUT.exists()
OUT.mkdir()
sys.path.insert(0, str(FIX))
sys.path.insert(0, str(Path(__file__).resolve().parent))
import sim466 as S  # noqa: E402
import sim472 as L  # noqa: E402

manifest = json.loads((FIX/'MANIFEST466.json').read_text())['files']
for name in ('sim466.py', 'launch448_466.py', 'receiver466.py'):
    assert hashlib.sha256((FIX/name).read_bytes()).hexdigest() == manifest[name]['sha256'], name
S.load463 = lambda world, Q0, faults=None, outcome=('complete', 0): L.load472(world, Q0, H472, faults, outcome)
GLOB = "'relay463-05-gallery-a*-run*'"
for name, sub in (('test466.py', 'runs-a'), ('test466b.py', 'runs-b'), ('test466c.py', 'runs-c')):
    path = FIX/name
    data = path.read_bytes()
    assert hashlib.sha256(data).hexdigest() == manifest[name]['sha256'], name
    text = data.decode()
    if name == 'test466.py':
        assert text.count(GLOB) == 1
        text = text.replace(GLOB, GLOB.replace('463', '472'))
    sys.argv = [str(path), str(Q0), str(OUT/sub)]
    with open(OUT/(sub+'.stdout'), 'x') as f, contextlib.redirect_stdout(f):
        exec(compile(text, str(path), 'exec'), {'__name__': '__main__', '__file__': str(path)})
    print(name, 'done', file=sys.stderr)
