"""Launch csd-native-counting-build02 once, under guard_csd02. Prepared only; root runs it on CSD.

  --check   verify every input and print what the inputs manifest would hold; writes nothing, starts nothing
  --launch  the same checks, then write csd-counting02-inputs.json (open 'x') and run the guard once

Never retries, never waits, never reuses a job, source or inputs name. The inputs manifest mirrors
build844's guard inputs (csd-native844-inputs.json 3b54974b): the guard files, the Q inputs, the Cargo
configs, the six toolchain binaries, every OLD workspace file and every engine/repo-view row on
work/tree, plus the bundle manifest and the four staged replacements.
"""
import hashlib
import json
from pathlib import Path
import os
import subprocess
import sys

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
JOB_NAME = 'csd-native-counting-build02'
JOB = D/'jobs'/JOB_NAME
SOURCE = D/'work/native-counting02'
INPUTS = Q/'csd-counting02-inputs.json'
OLD = D/'work/review-repair313/workspace'
HELPER = Q/'csd-counting02-build.py'
HELPER_SHA = '45d2ab26af048bd00880cdcc85baf37da10a7bac78e762ef1ea0ed3ada6b2f99'
PINNED = {
    Q/'csd-counting02-counting.rs': '21c5043a3339cfadb10024a2e8227a62a0b06674139eace1a0d9029a4e1aa0db',
    HELPER: HELPER_SHA,
    Q/'csd-counting02-bundle03-manifest.json': 'a5210650e0f8dc18860dfca9b2d06857a67588d6715fb04d990b7b8a39b1ee21',
    Q/'csd-counting02-the_bump.rs': '90b51c2b30877911bbb5f5e14ee810268e14efb24e7c4e2a55f7f7e128c06ee7',
    Q/'csd-counting02-loop_sums.rs': 'a8d62f560321ea6ebea3d3859acd9a3ce3e5dde4375ec0ab42c1a1ba114d61ef',
    Q/'csd-counting02-mod.rs': '4de1e4a56fde5c11ac3d6a65e772d09ec33db4f5757d817aff370799d3423578',
    Q/'csd-counting02-boundary.rs': '4b689791748132bd14c3929d191bb78ae7752bf9c3332beb22f774931873e932',
    Q/'csd-native844-base761.py': '32bf6d9e08c1f21f2445debfd46ba090e0374007e4da5516310cb122774099e5',
    Q/'csd-native844-source760.json': '5afe200ee13dec271e924fb60386554ebc28af4cbacfeb0dea99e4418bf741c0',
    Q/'csd-native844-narration_units.json': '995e25c7788943deff65c4d84050d8a17edc3c2c3498be0d665d7ac5b661357a',
    Q/'csd-native844-Cargo.toml': 'be1426d32dad7d61bafa84b5811bf0192a31943f70735e589a714b5db2070c10',
    Q/'csd-native844-config.toml': '79fb8766d1f41365d64066e5b2e68b0fb89c50015425d109b155ccaf8eff2716',
    Q/'csd-native844-engine-budget.toml': '116b317b92b8da9eb7584812626d1988045eb03ee7b7aaed0f35deb49cb297c0',
    D/'guard_csd02.py': 'f1cae1dbc80075311b0e1a2abf8637439d2d8d0edbc643e771e2eefe2b3bdfb6',
    D/'inputs/repo/docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py':
        '1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097',
    D/'inputs/repo/narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/run_bounded_phase_oliver_owner48g_target14g_20261102.py':
        '819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da',
    D/'qualification01/owned-process-stop02.py': 'a2fe7985bf76a1982e8d08b3dfb23354d2f60b322b95ed82ca83fc5e39a09236',
    D/'work/.cargo/config.toml': '79fb8766d1f41365d64066e5b2e68b0fb89c50015425d109b155ccaf8eff2716',
    Path.home()/'.cargo/config.toml': '14ae6b73244bde60c647383e9c4f6321fa575e7bb8ad75d6916a7adfefe0b4e6',
    # Template provenance (M1): build844's guard inputs bind csd-native844-build.py 7932fe78.
    Q/'csd-native844-build.py': '7932fe7803f311274ae850bc1c79a0d36df39db37aa579deda95e1e93ab11c9a',
    Q/'csd-native844-inputs.json': '3b54974babc3d17763d89040719c014a4999fb0542af3fa43d83636538e9979f',
    D/'jobs/csd-native-build844/preflight844.json': '37a28c9c492f27a5c491a01e877589fc1202a628546ecc0e08cc40492cdf7a02',
}
TOOLS = Path.home()/'.rustup/toolchains/nightly-2026-08-31-x86_64-unknown-linux-gnu/bin'
TOOL_SHA = {
    'cargo': 'b854c9424332416075760b8b5fc8a43254e8eea8f38f14a57deea6126810f1b9',
    'cargo-rch-real': '19008157a0089896bc835c5ceb0b5bfbe26e70cdd5579c32bd09538fb68c0b76',
    'rustc': '49f9fa06f3e2b93aba85273433fef67b984ddb4cacb4500de56e85f70ec2e339',
    'cargo-clippy': '92708a2623b9186bd8d40e6f3ddac686b6fd541bd052584fb4343dfcfe92dcd8',
    'clippy-driver': '2cd2fc266f83e81a1fc9bd46528c5be8db6ec7d940f67872123feef4a993a427',
    'rustfmt': '016e3712bb170692190f20ce3356646d6d2c59343b81516de66af7c5fd2e5e7c',
}
UNPINNED = [D/'jobs/allocation-baseline.json']   # the guard's own record, bound by hash as build844 did


def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def read(p):
    return json.loads(Path(p).read_text())


def success(p):
    d = read(p)
    assert d.get('admitted') and d.get('finished') and d.get('exit') == 0, p
    assert d['inputs_sha256'] == d['final_inputs_sha256'], p
    assert not any(d.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), p
    return d


def rows():
    assert HELPER_SHA != '@@' + 'HELPER_SHA@@', 'launcher not frozen'
    assert not JOB.exists() and not INPUTS.exists() and not SOURCE.exists(), 'Never duplicate or overwrite a job'
    out = {}
    for p, digest in PINNED.items():
        assert p.is_file() and not p.is_symlink() and sha(p) == digest, str(p)
        out[str(p)] = digest
    for name, digest in TOOL_SHA.items():
        p = TOOLS/name
        assert p.is_file() and sha(p) == digest, str(p)
        out[str(p)] = digest
    for p in UNPINNED:
        out[str(p)] = sha(p)
    r844 = success(D/'jobs/csd-native-build844/resource-receipt.json')
    assert r844['inputs_sha256'] == PINNED[Q/'csd-native844-inputs.json']
    bound = {x['path']: x['sha256'] for x in read(Q/'csd-native844-inputs.json')['files']}
    assert bound[str(Q/'csd-native844-build.py')] == PINNED[Q/'csd-native844-build.py']
    manifest = read(Q/'csd-native844-source760.json')['files']
    tree = 0
    for rel, digest in manifest.items():
        if rel.startswith(('engine/', 'repo-view/')):
            p = D/'work/tree'/rel
            assert p.is_file() and not p.is_symlink() and sha(p) == digest, str(p)
            out[str(p)] = digest
            tree += 1
    assert tree == 4125
    old = [p for p in OLD.rglob('*') if p.is_file()]
    assert len(old) == 72 and not any(p.is_symlink() for p in old)
    for p in old:
        out[str(p)] = sha(p)
    out[str(Path(__file__).resolve())] = sha(Path(__file__).resolve())
    return out


def main():
    assert sys.argv[1:] in (['--check'], ['--launch']), 'usage: --check | --launch'
    inputs = rows()
    st = os.statvfs(D)
    free = st.f_bavail * st.f_frsize / 2**30
    print(json.dumps(dict(rows=len(inputs), free_gib=round(free, 1), job=JOB_NAME, source=str(SOURCE),
                          inputs=str(INPUTS), helper_sha256=HELPER_SHA)))
    if sys.argv[1] == '--check':
        print('check passed; nothing written, nothing started')
        return
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=k, sha256=v) for k, v in sorted(inputs.items())]), f, indent=2)
    subprocess.run(['/usr/bin/python3', '-I', '-B', str(D/'guard_csd02.py'), '--phase', 'native',
                    '--job', JOB_NAME, '--inputs', str(INPUTS), '--',
                    '/usr/bin/python3', '-I', '-B', str(HELPER), '--job', JOB_NAME], check=True)
    success(JOB/'resource-receipt.json')
    q = read(JOB/'qualification.json')
    print(json.dumps(dict(binary_sha256=q['binary_sha256'], tests_passed=q['tests_passed'], facts_sha256=q['facts_sha256'])))
    print('counting02 build qualified; narration rebind, render, pictures and master gates remain open')


if __name__ == '__main__':
    main()
