"""task455 trial b: master350_455.py unchanged except that each refusal also names the 451 line that refused it.
Reads master350_455.py, applies one asserted replacement in memory, runs it. Usage as master350_455.py."""
import hashlib
from pathlib import Path
import sys

src = (Path(__file__).resolve().parent/'master350_455.py').read_text()
old = """        results[variant] = dict(outcome='refused', error='%s: %s' % (type(error).__name__,
                                str(error).replace(str(OUTDIR), 'T')[:200]))"""
new = """        import traceback
        frames = [f for f in traceback.extract_tb(error.__traceback__) if f.filename == m.__file__]
        last = frames[-1] if frames else None
        results[variant] = dict(outcome='refused', error='%s: %s' % (type(error).__name__,
                                str(error).replace(str(OUTDIR), 'T')[:200]),
                                v451_line=last.lineno if last else None,
                                v451_function=last.name if last else None,
                                v451_text=V451_LINES[last.lineno - 1].strip() if last else None)"""
assert src.count(old) == 1
src = src.replace(old, new)
src = src.replace("results = {}\n", "results = {}\nV451_LINES = (Q0/'csd-final-verify451.py').read_text().splitlines()\n", 1)
print('# base master350_455.py sha256', hashlib.sha256((Path(__file__).resolve().parent/'master350_455.py').read_bytes()).hexdigest(),
      file=sys.stderr)
exec(compile(src, str(Path(__file__).resolve().parent/'master350_455.py'), 'exec'), {'__name__': '__main__'})
