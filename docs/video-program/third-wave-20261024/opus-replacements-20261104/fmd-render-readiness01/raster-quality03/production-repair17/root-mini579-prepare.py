"""Freeze a new guarded native10 import. Does not launch or claim acceptance."""
import argparse
import ast
import hashlib
import importlib.util
import json
import os
import re
from pathlib import Path

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
TRANSPORT_SHA = '5739b6901d895b02668f6642437260e02afea78078d55a55347ed9ae39c9b969'
LIB_SHA = 'ba337aca143316748561c4300d5774386387690b49052e72d29a04b103dd244f'

def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--manifest', type=Path, required=True)
    p.add_argument('--manifest-sha', required=True)
    p.add_argument('--attempt', type=int, required=True)
    a = p.parse_args()
    helper = Path(__file__).resolve()
    assert helper.parent == Q and a.manifest.parent == Q
    assert not a.manifest.is_symlink() and sha(a.manifest) == a.manifest_sha
    assert 1 <= a.attempt <= 999
    transport, library, guard = Q/'root-mini576-transport.py', Q/'csd-css352-import371.py', D/'guard_csd02.py'
    assert sha(transport) == TRANSPORT_SHA and sha(library) == LIB_SHA
    spec = importlib.util.spec_from_file_location('native576', transport)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    manifest = json.loads(a.manifest.read_text())
    module.validate(manifest)
    # The receiver will verify the full imported closure before accepting it.
    # This preparation only checks the manifest shape, local dependencies and space.
    job = 'csd-mini-import576-10-a'+str(a.attempt)
    inputs = Q/(job+'-inputs.json')
    assert not os.path.lexists(D/'jobs'/job) and not os.path.lexists(inputs)
    pins = None
    for node in ast.parse(guard.read_text()).body:
        if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'PINS' for t in node.targets):
            pins = ast.literal_eval(node.value)
    assert isinstance(pins, dict) and len(pins) == 2
    files = {helper, a.manifest, transport, library, guard,
             Path('/usr/bin/python3').resolve(), Path('/usr/bin/timeout').resolve()}
    for name, digest in pins.items():
        assert not Path(name).is_absolute() and '..' not in Path(name).parts
        path = D/name
        assert re.fullmatch('[0-9a-f]{64}', digest) and sha(path) == digest
        files.add(path)
    stop = D/'qualification01/owned-process-stop02.py'
    assert sha(stop) == 'a2fe7985bf76a1982e8d08b3dfb23354d2f60b322b95ed82ca83fc5e39a09236'
    files.add(stop)
    st = os.statvfs(D)
    free = st.f_bavail*st.f_frsize
    assert free >= 10*(1 << 30) and free-manifest['total_bytes'] >= 8*(1 << 30)
    rows = [dict(path=str(f), sha256=sha(f)) for f in sorted(files)]
    command = ['/usr/bin/python3', '-I', '-B', str(guard), '--phase', 'native', '--job', job,
               '--inputs', str(inputs), '--', '/usr/bin/timeout', '14400', '/usr/bin/python3',
               '-I', '-B', str(transport), 'receive', '--manifest', str(a.manifest),
               '--job', job, '--inputs', str(inputs)]
    with inputs.open('x') as f:
        json.dump(dict(files=rows), f, indent=2)
        f.write('\n')
    print(json.dumps(dict(job=job, inputs=str(inputs), inputs_sha256=sha(inputs),
                         manifest_sha256=sha(a.manifest), guard_command=command,
                         launched=False, scope='Preparation only; guard rechecks resource admission and all frozen inputs.'), indent=2))

if __name__ == '__main__':
    main()
