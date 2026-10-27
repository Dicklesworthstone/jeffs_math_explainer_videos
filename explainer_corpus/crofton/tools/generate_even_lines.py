"""Retain the oracle's line stream after its pin experiment, without rerunning it."""
from pathlib import Path
import argparse
import hashlib
import importlib.util
import json
import numpy as np

parser = argparse.ArgumentParser()
parser.add_argument('--out', type=Path, required=True)
parser.add_argument('--receipt', type=Path, required=True)
args = parser.parse_args()
if args.out.exists() or args.receipt.exists():
    raise SystemExit('Refusing to replace retained data or receipt')
crate = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('crofton_oracle', crate/'oracle.py')
oracle = importlib.util.module_from_spec(spec)
spec.loader.exec_module(oracle)
rng = np.random.Generator(np.random.PCG64DXSM(oracle.SEED_LINES))
# Each pin pair consumes four binary64 uniforms. Check jump equivalence using
# raw RNG state only; no previous pin geometry or picture qualification is rerun.
rng.bit_generator.advance(4*oracle.LINES)
control = np.random.Generator(np.random.PCG64DXSM(oracle.SEED_LINES))
control.random(4*oracle.LINES)
if rng.bit_generator.state != control.bit_generator.state:
    raise ValueError('PCG jump does not reproduce the pin-prefix state')
start_state = rng.bit_generator.state
lines = [oracle.even_line(rng) for _ in range(oracle.LINES)]
text = f'# PCG64DXSM seed={oracle.SEED_LINES} skip_uniforms={4*oracle.LINES}\n'
text += ''.join('\t'.join(repr(float(x)) for p in line for x in p)+'\n' for line in lines)
args.out.parent.mkdir(parents=True, exist_ok=True)
with args.out.open('x') as f:
    f.write(text)
receipt = dict(seed=oracle.SEED_LINES, algorithm='NumPy PCG64DXSM', numpy=np.__version__,
    skipped_uniforms=4*oracle.LINES, jump_state_equals_consumed_state=True,
    start_state=start_state, end_state=rng.bit_generator.state, lines=len(lines),
    columns=['px','py','qx','qy'], meaning='Two defining points of each infinite line, not random pins.',
    data_sha256=hashlib.sha256(args.out.read_bytes()).hexdigest(),
    retained_pin_inputs_sha256=hashlib.sha256((crate/'data/pins-20261025.tsv').read_bytes()).hexdigest(),
    oracle_sha256=hashlib.sha256((crate/'oracle.py').read_bytes()).hexdigest(),
    generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
with args.receipt.open('x') as f:
    json.dump(receipt,f,indent=2); f.write('\n')
print({k:v for k,v in receipt.items() if k not in ['start_state','end_state']})
