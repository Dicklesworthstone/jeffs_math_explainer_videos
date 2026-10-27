"""Retain the actual oracle's seeded inputs; native Rust computes their crossings.

Run with python3 -I. Refuses to overwrite. Decimal repr round-trips each binary64
coordinate; this is input data, not a precomputed table of crossing answers.
"""
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
crate = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('crofton_oracle', crate / 'oracle.py')
oracle = importlib.util.module_from_spec(spec)
spec.loader.exec_module(oracle)
rng = np.random.Generator(np.random.PCG64DXSM(oracle.SEED_LINES))
lines = [oracle.pin_line(rng) for _ in range(oracle.LINES)]
text = '# PCG64DXSM seed=' + str(oracle.SEED_LINES) + '\n'
text += ''.join('\t'.join(repr(float(x)) for p in line for x in p) + '\n' for line in lines)
if args.out.exists() or args.receipt.exists():
    raise SystemExit('Refusing to overwrite retained data or receipt')
args.out.parent.mkdir(parents=True, exist_ok=True)
args.out.write_text(text)
receipt = dict(seed=oracle.SEED_LINES, algorithm='NumPy PCG64DXSM', numpy=np.__version__, pairs=len(lines),
    columns=['px','py','qx','qy'], coordinate_format='decimal binary64 round-trip',
    data_sha256=hashlib.sha256(args.out.read_bytes()).hexdigest(),
    oracle_sha256=hashlib.sha256((crate/'oracle.py').read_bytes()).hexdigest(),
    generator_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
args.receipt.write_text(json.dumps(receipt, indent=2)+'\n')
print(json.dumps(receipt))
