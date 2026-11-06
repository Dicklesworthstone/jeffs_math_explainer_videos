"""Read-only FMD sender and original-guard-only CSD receiver."""
import importlib.util
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('wire', HERE/'review-transfer-wire12.py')
wire = importlib.util.module_from_spec(spec)
spec.loader.exec_module(wire)
FMD = Path('/data/projects/jmev-fmd-render-qualification-20261106-1111')
CSD = Path('/data/projects/jmev-stroke-probe-20261106')

def read(p):
    return json.loads(p.read_text())

def main():
    assert len(sys.argv) == 3
    mode, expected = sys.argv[1:]
    assert len(expected) == 64 and all(c in '0123456789abcdef' for c in expected)
    if mode == '--send':
        assert HERE == FMD/'qualification01/raster-quality03'
        job = FMD/'jobs/fmd-raster-transfer12'
        receipt = read(job/'resource-receipt.json')
        assert receipt.get('finished') and receipt['exit'] == 0 and not receipt.get('resource_stop')
        assert receipt['inputs_sha256'] == receipt['final_inputs_sha256']
        manifest = FMD/'fmd-raster-transfer12-inputs.json'
        assert wire.digest(manifest.read_bytes()) == receipt['inputs_sha256']
        for row in read(manifest)['files']:
            assert wire.digest(Path(row['path']).read_bytes()) == row['sha256']
        wire.send(job/'production-inputs-raster12', sys.stdout.buffer, expected)
    else:
        assert mode == '--receive' and HERE == CSD/'qualification01/raster-quality12'
        job = CSD/'jobs/csd-raster-review-receive12'
        spec = importlib.util.spec_from_file_location('worker', HERE/'csd-final-chapter12.py')
        worker = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(worker)
        receipt = read(job/'resource-receipt.json')
        assert receipt['admitted'] and 'exit' not in receipt and receipt['policy'] == worker.POLICY
        assert receipt['command'] == ['/usr/bin/python3','-I','-B',str(Path(__file__).resolve()),mode,expected]
        worker.check_media_paths()
        rows = wire.receive(sys.stdin.buffer,CSD/'work/incoming-review12',expected)
        with (job/'receive-summary.json').open('x') as f:
            json.dump({'transfer_sha256':expected,'files':rows,'scope':'Exact bounded transport only; semantic review import remains required.'},f,indent=2)
        print('Review package received; semantic import remains.')

if __name__ == '__main__':
    main()
