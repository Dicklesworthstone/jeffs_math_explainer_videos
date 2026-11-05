#!/usr/bin/env python3
"""Replay this review's claims about root's remote build from local copies only.

Read-only. It uses no SSH, compiles nothing, starts no job and writes nothing in the repository. It
runs one print-only `facts` of the existing stage09 binary and the stdlib oracle. A `found-` line
reproduces a finding; every other line is a check that came back clean. From the repository root:

    python3 -I -B <bundle>/checks/check_remote_build.py
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve()
R = HERE.parents[6]
P = HERE.parents[2]
E = R / 'narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104'
O = E / 'remote-mac-mini-max01'
S = E / 'final-picture-repair01'
C = S / 'workspace/explainer_corpus/logarithm_mysteries'
B = R / 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
H = P / 'logarithm-final-audio-rebind01/rebind_audio.py'
GIB = 1024 ** 3
ROOT_REMOTE_CONFIG = '79fb8766d1f41365d64066e5b2e68b0fb89c50015425d109b155ccaf8eff2716'  # root, Mail 5287
results = []


def check(name, ok, detail):
    results.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'} {name}: {detail}", flush=True)


def sha(path):
    with open(path, 'rb') as handle:
        return hashlib.file_digest(handle, 'sha256').hexdigest()


def rows(path):
    return {row['path']: row['sha256'] for row in json.loads(path.read_text())['files']}


def transfer_lists():
    return {'/' + line for name in ('source', 'support', 'launch', 'audio')
            for line in (O / f'{name}-files01.txt').read_text().splitlines() if line}


def peak_growth(job):
    data = json.loads((B / job / 'resource-receipt.json').read_text())
    start = data['admission']
    return max(max(0, s['target_bytes'] - start['target_bytes']) + max(0, s['work_bytes'] - start['work_bytes']) for s in data['samples'])


def guard_checks():
    pins = json.loads((O / 'guard-pins01.json').read_text())['files']
    check('guard-pins', all(sha(row['path']) == row['sha256'] for row in pins) and len(pins) == 2, 'both pinned guard files match their hashes')
    run = subprocess.run([sys.executable, '-I', '-B', str(O / 'guard01.py')], capture_output=True, text=True)
    check('guard-host', run.returncode != 0 and 'restricted to mac-mini-max' in run.stderr, 'on another host guard01.py refuses before doing anything')
    path = P / 'engine-secondary-validation01/guard_owner56g_target20g.py'
    spec = importlib.util.spec_from_file_location('preserved_guard', path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    guard = module.load()
    check('guard-thresholds', (guard.ADMISSION_GIB['native'], guard.ADMISSION_GIB['asr'], guard.STOP_FREE_GIB, guard.CAPS['native'],
                               guard.WORK_CAP_GIB, guard.TARGET_GROWTH_CAP_GIB) == (10, 12, 8, 4, 56, 20), 'native 10, ASR 12, stop 8, per job 4, work 56, target growth 20')
    with tempfile.TemporaryDirectory() as folder:
        lane = Path(folder)
        guard.ROOT, guard.WORK, guard.TARGET = lane, lane / 'jobs', lane / 'target'
        missing = guard.sample()
        for name, size in (('jobs', 1), ('target', 3)):
            (lane / name).mkdir()
            (lane / name / 'file').write_bytes(bytes(size * 1024 ** 2))
        now = guard.sample()
    check('guard-siblings', missing['target_bytes'] == 0 and 1024 ** 2 <= now['work_bytes'] < 2 * 1024 ** 2 and 3 * 1024 ** 2 <= now['target_bytes'] < 4 * 1024 ** 2,
          'overrides take effect when sampled, a missing target counts as 0, and the siblings count apart')
    source = (R / 'narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/run_bounded_phase_oliver_owner48g_target14g_20261102.py').read_text()
    check('guard-baseline', 'baseline_path = WORK / "allocation-baseline.json"' in source and '(ROOT / "expanded-native-lane.lock")' in source
          and 'provenance' not in source and 'local-baseline' not in (O / 'guard01.py').read_text(),
          'baseline and lock follow WORK and ROOT into O, and the guard never reads the provenance copy')


def source_checks(remote, local, lists):
    missing = [p for p in remote if not Path(p).is_file()]
    differ = [p for p, digest in remote.items() if p not in missing and sha(p) != digest]
    check('source-current', len(remote) == 4308 and not missing and not differ, '4308 rows, all equal to the current local bytes')
    check('source-against-freeze', not [p for p in set(remote) & set(local) if remote[p] != local[p]], 'no row shared with the local build freeze changed')
    only_local = [p for p in set(local) - set(remote) if '/target/' not in p]
    check('source-dropped', len(only_local) == 8 and len(set(local) - set(remote)) == 67,
          '67 local-only rows: 59 old target artifacts and 8 others, none named by build01.sh except the config in found-config')
    check('source-transferred', not set(remote) - lists, 'every manifest row is in a transfer list')
    text = (O / 'build01.sh').read_text()
    W = Path(next(p for p in remote if p.endswith('/engine-budget.toml'))).parent
    named = [W / 'engine-budget.toml', W / 'logarithm-stage09-original-Cargo.lock', R / 'rustfmt.toml', O / 'build01.sh', C / 'Cargo.toml', C / 'Cargo.lock', C / 'oracle.py',
             P / 'engine-geometry-validation02/verify_patched_lock.py']
    chapters = ('draw', 'complex_turns', 'lifting_loops', 'unreachable', 'what_it_forgets', 'undoing_growth')
    named += [C / f'src/chapters/{name}.rs' for name in chapters]
    check('source-named', all(str(p) in remote for p in named) and all(name in text for name in chapters), 'every file build01.sh names is frozen')
    sealed = {'draw': 'logarithm-final-middle-repair01/staged/src/chapters', 'complex_turns': 'logarithm-final-middle-repair01/staged/src/chapters',
              'lifting_loops': 'logarithm-final-middle-repair01/staged/src/chapters', 'unreachable': 'logarithm-final-picture-repair01',
              'what_it_forgets': 'logarithm-final-picture-repair01', 'undoing_growth': 'logarithm-final-growth-repair01'}
    check('source-repairs', all(sha(C / f'src/chapters/{name}.rs') == sha(P / folder / f'{name}.rs') for name, folder in sealed.items()),
          'the six chapters equal their repair files byte for byte')
    return W


def build_checks(W):
    text = (O / 'build01.sh').read_text()
    check('build-pinned', text.count('rustup run nightly-2026-08-31') == 4 and text.count('--locked --offline') == 3 and 'test ! -e "$O/target"' in text
          and 'test "${CARGO_TARGET_DIR:-}" = "$O/target"' in text, 'pinned nightly, locked and offline cargo, and a target that must start absent')
    release, debug = peak_growth('complex-stage06-optimized-build01'), peak_growth('logarithm-stage09-ws-build02')
    check('build-growth', release < 0.8 * GIB and debug < 1.4 * GIB and release + debug < 2.2 * GIB,
          f'closest real builds grew {release / GIB:.2f} GiB (release test and build) and {debug / GIB:.2f} GiB (debug with clippy), against the 4 GiB cap')
    same = all(sha(C / name) == sha(W / 'logarithm-stage09' / name) for name in ('src/facts.rs', 'src/model.rs', 'oracle.py'))
    native = subprocess.run([str(W / 'logarithm-stage09-build01-binary'), 'facts'], capture_output=True, text=True, timeout=60)
    oracle = subprocess.run([sys.executable, '-I', '-B', str(C / 'oracle.py')], capture_output=True, text=True, timeout=60)
    check('build-facts', same and native.returncode == oracle.returncode == 0 and native.stdout == oracle.stdout and len(native.stdout.splitlines()) == 102,
          'facts.rs, model.rs and oracle.py are unchanged since stage09, and its binary and the oracle match byte for byte over 102 rows')


def local_checks():
    names = [n for n in os.listdir(B) if n.startswith('logarithm-final-picture') or 'remote' in n]
    check('local-no-duplicate', not names and not (S / 'logarithm_mysteries-release01').exists() and not [n for n in os.listdir(S) if n.endswith('.exit')],
          'no local final-picture job, binary or exit file exists')


def findings(remote, local, lists):
    units = str(C / 'narration_units.json')
    audio = (S / 'audio-after-build01.py').read_text()
    rebind = H.read_text()
    check('found-units', units in remote and units not in local and "assert mutable not in rows, 'Rebind output cannot be a frozen input'" in audio
          and 'os.replace(temporary, workspace / UNITS)' in rebind and 'for p, digest in rows.items()' in audio,
          'narration_units.json is frozen for the remote build, and the rebind rewrites it; the local chain excludes it and refuses it')
    others = [str(S / name) for name in ('spans01.json', 'holds-bound01.json', 'rebind-report01.json', 'narration_units-before-rebind01.json')]
    others.append(str(C / 'docs/narration-direction-map-oliver17.json'))
    check('found-units-only', not set(others) & set(remote), 'no other rebind output is a frozen row')
    config = str(R / '.cargo/config.toml')
    check('found-config', config in local and config not in remote and config not in lists and local[config] == sha(config) == ROOT_REMOTE_CONFIG,
          "the cargo config is in the local freeze but not the remote manifest or transfer lists, and the local bytes equal the hash root read on the remote")
    text = (O / 'build01.sh').read_text()
    check('found-binary', text.index('cp -p "$CARGO_TARGET_DIR/release') < text.index('cargo clippy') and 'shasum' not in text and 'sha256' not in text,
          'the binary lands in O before clippy and the facts check, and no job file records its hash')
    wired = [name for name in ('audio-after-build01.py', 'render-master01.py') if 'guard_owner56g_target20g.py' in (S / name).read_text()]
    check('found-guard-wiring', wired == ['audio-after-build01.py', 'render-master01.py'], 'both local continuations hard-code the original guard, not guard01.py')
    prepared = E / 'production-audio-stage09-01/prepared02'
    copy = json.loads((S / 'initial-copy.json').read_text())
    paths = [H, S / 'initial-copy.json', S / 'assemble01.py', E / 'combined-repair-stage09/bound-plan/hold-audit.json']
    paths += [Path(copy['source']) / row['path'] for row in copy['files']]
    paths += [S / 'workspace' / row['path'] for row in copy['files'] if not row['path'].endswith('/narration_units.json')]
    paths += [p for p in prepared.rglob('*') if p.is_file()]
    for unit in json.loads((prepared / 'spans.json').read_text())['units']:
        paths += [R / unit['wav'], R / unit['generation_receipt']]
    wanted = {str(p) for p in paths}
    check('next-step-transferred', len(wanted) == 276 and not wanted - lists, 'all 276 inputs of the rebind and assembly are in the transfer lists')


def main():
    remote, local, lists = rows(O / 'build-inputs01.json'), rows(S / 'build-inputs01.json'), transfer_lists()
    guard_checks()
    W = source_checks(remote, local, lists)
    build_checks(W)
    local_checks()
    findings(remote, local, lists)
    print(f"RESULT {'PASS' if all(results) else 'FAIL'}, {results.count(False)} failed")
    return 0 if all(results) else 1


if __name__ == '__main__':
    sys.exit(main())
