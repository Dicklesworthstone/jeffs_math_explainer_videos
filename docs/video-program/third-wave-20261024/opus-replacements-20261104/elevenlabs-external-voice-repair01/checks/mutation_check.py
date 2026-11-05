"""Mutation check: the synthetic suite must fail on each weakened copy of the patch.

READ-ONLY on the repository; writes mutant tool mirrors only into --work.
M0 runs the unpatched canonical tool in the patched slot (the defect itself).

Usage, from the repository root:
    python3 -I -B <bundle>/checks/mutation_check.py --work NEW_DIR
Exit 0 every mutant killed, 1 a mutant survived, 2 usage.
"""
import argparse
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

BUNDLE = Path(__file__).resolve().parents[1]
MUTANTS = {
    'M1 no mixed-voice guard': ('    voices = set()\n', '    return None\n    voices = set()\n'),
    'M2 no v2 request check': ('        if v2:\n            # The request', '        if False:\n            # The request'),
    'M3 prefix label acceptance': ("    known = isinstance(source, str) and source in EXTERNAL_UI_VOICES\n",
                                   "    known = isinstance(source, str) and source.startswith('ElevenLabs UI, ')\n"
                                   "    source = 'ElevenLabs UI, Frederick Surrey' if known and 'Oliver' not in source else source\n"),
    'M4 no per-record voice check': ("or row.get('voice') != voice for row in records)", "for row in records)"),
    'M5 receipt voice substring': ("receipt.get('voice') != voice:", "receipt.get('voice') not in voice:"),
    'M6 legacy accepts any schema': ("if source == LEGACY_EXTERNAL_UI_SOURCE and 'schema' not in receipt:",
                                     "if source == LEGACY_EXTERNAL_UI_SOURCE:"),
    'M0 canonical base as patched': (None, None),
}


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--work', type=Path, required=True)
    parser.add_argument('--base-tools', type=Path, default=Path('tools'))
    args = parser.parse_args()
    if args.work.exists():
        print(f'usage: {args.work} exists', file=sys.stderr)
        return 2
    work, base = args.work.resolve(), args.base_tools.resolve()
    work.mkdir(parents=True)
    patched = (BUNDLE / 'patched/tools/coherent_narration.py').read_text()
    env = dict(os.environ, TMPDIR=str(work))
    survived = []
    for label, (old, new) in MUTANTS.items():
        tools = work / re.sub(r'\W+', '-', label) / 'tools'
        tools.mkdir(parents=True)
        for name in ('coherent_narration.py', 'narration_cache.py', 'narration_delivery.py'):
            shutil.copyfile(base / name, tools / name)
        if old is not None:
            if patched.count(old) != 1:
                print(f'{label}: mutation site not found exactly once', file=sys.stderr)
                return 1
            (tools / 'coherent_narration.py').write_text(patched.replace(old, new))
            shutil.copyfile(BUNDLE / 'patched/tools/narration_delivery.py', tools / 'narration_delivery.py')
        r = subprocess.run([sys.executable, '-I', '-B', str(BUNDLE / 'tests/test_external_voice.py'), str(base), str(tools)],
                           env=env, text=True, capture_output=True, timeout=300, cwd=work)
        bad = sorted(set(re.findall(r'^(?:FAIL|ERROR): (\w+)', r.stderr, re.M)))
        if r.returncode == 0 or not bad:
            survived.append(label)
        print(f'{label}: {"killed" if bad else "SURVIVED"} by {len(bad)} tests: {", ".join(bad)}')
    print('ALL MUTANTS KILLED' if not survived else 'SURVIVED: ' + '; '.join(survived))
    return 0 if not survived else 1


if __name__ == '__main__':
    sys.exit(main())
