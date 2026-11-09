"""Activate the source already exercised by the compiled texture wheel.

Keep this exact integration separate from a validation receipt: the workflow
reruns native/wheel checks on the integrated main commit after publication.
"""
import hashlib
from pathlib import Path
import runpy
import subprocess

runpy.run_path('scripts/apply_surface_textures.py', run_name='__main__')
patch = 'scripts/patches/texture-bridge-acceptance.patch'
if subprocess.run(['git', 'apply', '--reverse', '--check', patch], capture_output=True).returncode:
    subprocess.run(['git', 'apply', '--check', '--whitespace=error-all', patch], check=True)
    subprocess.run(['git', 'apply', '--whitespace=error-all', patch], check=True)
paths = ['crates/fmn-render/src/retained.rs', 'crates/fmn-render/src/retained_texture_tests.rs',
         'crates/fmn-library/src/solids.rs', 'crates/fmn-library/tests/textured_mesh_records.rs',
         'crates/fmn-python/src/lib.rs', 'crates/fmn-python/src/portal_texture.rs',
         'crates/fmn-conformance/tests/e2e_scenarios.rs']
subprocess.run(['rustfmt', '--edition', '2024', '--config', 'skip_children=true', *paths], check=True)
expected = {
    'crates/fmn-render/src/retained.rs': '3dfb69931e30a2f6621eaba887bd4b9baf932ad3d541374085d645ce900d0ea5',
    'crates/fmn-library/src/solids.rs': 'd3b7acf7c482b308dd97ccc1f01cad68208cb3ad9ba21bbdb2272af7fcd669ad',
    'crates/fmn-python/src/portal_texture.rs': 'bd2bb176b502bd0d8255e6d0b778a9281951740298f520bdaf8a09fa5599ef72',
}
for name, digest in expected.items():
    if hashlib.sha256(Path(name).read_bytes()).hexdigest() != digest:
        raise RuntimeError('texture authority changed since the compiled native proof: ' + name)
print('Native texture authorities match the compiled wheel source; integration is ready.')
