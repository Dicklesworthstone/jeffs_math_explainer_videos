"""Apply the bundle's patches to pinned canonical copies and run every check.

SYNTHETIC: the tests generate their own integer PCM; nothing is synthesised,
rendered, transcribed or heard, and no repository file is written.

Usage, from the repository root:
    python3 -I -B <bundle>/tests/run_tests.py --work NEW_SCRATCH_DIR

Steps:
  1. refuse unless the canonical tools still have the pinned base hashes;
  2. copy them into WORK/base/tools and WORK/applied/tools, apply patch/*.diff
     to the second with patch(1), and require the result to equal patched/tools;
  3. run the canonical tools/test_*.py suites against both mirrors;
  4. run tests/test_external_voice.py (base contrast + patched contract).
Exit codes: 0 all passed, 1 a check failed, 2 usage, 3 stale base pins.
"""
import argparse
import hashlib
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

BUNDLE = Path(__file__).resolve().parents[1]
BASE_PINS = {
    'coherent_narration.py': 'e0aa81f352cb2c847f53dacf1d37b7d1b6a44526102df7f79329278c278c48d0',
    'narration_delivery.py': 'c91914b89bed9e7aca71a649d403fa591bf40ec135f9c3d30f9d5e45c88422ba',
    'narration_cache.py': '47331efd3952af6d0e66971a1c8fddd08929f881d1f9b470bde223db7a01b0c0',
    'test_coherent_narration.py': '074bf1b2e5ce007e702bff3166eb3e6aea006f2f73a9b865740d74a833dfba01',
    'test_narration_delivery.py': 'bd45cb3d0285f287e6acaf71b0dda54cea6974b3db245c03a7bcba0830baf3fe',
}
PATCHES = ('coherent_narration.diff', 'narration_delivery.diff')
PATCHED = ('coherent_narration.py', 'narration_delivery.py')


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(cmd, env, log, cwd):
    result = subprocess.run(cmd, cwd=cwd, env=env, text=True, capture_output=True, timeout=600)
    log.write_text(result.stdout + result.stderr)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--work', type=Path, required=True, help='new scratch directory (must not exist)')
    parser.add_argument('--tools', type=Path, default=Path('tools'), help='canonical tools directory (default: tools)')
    args = parser.parse_args()
    if args.work.exists():
        print(f'usage: {args.work} exists; give a new directory', file=sys.stderr)
        return 2
    stale = [n for n, h in BASE_PINS.items() if not (args.tools / n).is_file() or digest(args.tools / n) != h]
    if stale:
        print('STALE: canonical base differs from the pins: ' + ', '.join(stale), file=sys.stderr)
        return 3
    work = args.work.resolve()
    (work / 'tmp').mkdir(parents=True)
    (work / 'logs').mkdir()
    env = dict(os.environ, TMPDIR=str(work / 'tmp'))
    failures = []

    def check(name, ok, detail=''):
        print(('PASS ' if ok else 'FAIL ') + name + (f'  [{detail}]' if detail else ''))
        if not ok:
            failures.append(name)

    mirrors = {}
    for label in ('base', 'applied'):
        tools = work / label / 'tools'
        tools.mkdir(parents=True)
        for name in BASE_PINS:
            shutil.copyfile(args.tools / name, tools / name)
        mirrors[label] = tools
    for diff in PATCHES:
        result = run(['patch', '-p1', '--batch', '--forward', '-d', str(work / 'applied'), '-i', str(BUNDLE / 'patch' / diff)],
                     env, work / 'logs' / f'apply-{diff}.txt', work)
        check(f'patch/{diff} applies to the pinned base', result.returncode == 0, result.stdout.strip().replace('\n', '; '))
    for name in PATCHED:
        same = (mirrors['applied'] / name).read_bytes() == (BUNDLE / 'patched/tools' / name).read_bytes()
        check(f'base + diff == patched/tools/{name}', same, digest(mirrors['applied'] / name)[:12])
    for name in BASE_PINS:
        if name not in PATCHED:
            check(f'{name} unchanged by the patch', digest(mirrors['applied'] / name) == BASE_PINS[name])

    counts = {}
    for label, tools in mirrors.items():
        result = run([sys.executable, '-I', '-B', '-m', 'unittest', 'discover', '-s', str(tools), '-p', 'test_*.py', '-v'],
                     env, work / 'logs' / f'canonical-tests-{label}.txt', work)
        ran = re.search(r'^Ran (\d+) tests?', result.stderr, re.M)
        counts[label] = int(ran.group(1)) if ran else -1
        check(f'canonical tools/test_*.py suites pass on {label}', result.returncode == 0 and ran is not None,
              f'{counts[label]} tests')
    check('same canonical test count before and after', counts['base'] == counts['applied'] > 0)

    result = run([sys.executable, '-I', '-B', str(BUNDLE / 'tests/test_external_voice.py'),
                  str(mirrors['base']), str(mirrors['applied'])], env, work / 'logs' / 'external-voice-tests.txt', work)
    ran = re.search(r'^Ran (\d+) tests?', result.stderr, re.M)
    check('synthetic external-voice regression suite', result.returncode == 0 and ran is not None,
          f'{ran.group(1) if ran else "?"} tests')
    for line in result.stderr.splitlines():
        if line.endswith(' ... ok') or ' ... FAIL' in line or ' ... ERROR' in line:
            print('    ' + line)
    print('logs: WORK/logs/ (canonical-tests-base.txt, canonical-tests-applied.txt, external-voice-tests.txt)')
    print('ALL PASS' if not failures else f'{len(failures)} FAILED: ' + '; '.join(failures))
    return 0 if not failures else 1


if __name__ == '__main__':
    sys.exit(main())
