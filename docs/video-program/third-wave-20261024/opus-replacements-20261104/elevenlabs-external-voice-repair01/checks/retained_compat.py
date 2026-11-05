"""Backward compatibility on retained real assemblies: canonical vs patched verify.

READ-ONLY on the repository; writes only a tool mirror into --work. For a
bounded sample of retained coherent assemblies whose exact script and unit
plan can be found by their bound sha256, run the canonical and the patched
coherent_narration.verify and require the same outcome (both pass, or the
same refusal). Verification is technical reconstruction, never listening.

Usage, from the repository root:
    python3 -I -B <bundle>/checks/retained_compat.py --work NEW_DIR [--oliver 24] [--native 12]
Exit 0 every sampled outcome agrees, 1 any differs, 2 usage.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys

BUNDLE = Path(__file__).resolve().parents[1]
ROOTS = ('narration_lab', 'explainer_corpus', 'docs')


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load(path, alias):
    spec = importlib.util.spec_from_file_location(alias, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def outcome(module, out, script, units, chapter):
    try:
        return 'pass' if module.verify(out, script, units, chapter=chapter) else 'false'
    except Exception as error:  # the comparison is the point; record any refusal
        return f'{type(error).__name__}: {error}'


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--work', type=Path, required=True)
    parser.add_argument('--oliver', type=int, default=24)
    parser.add_argument('--native', type=int, default=12)
    parser.add_argument('--patched-tools', type=Path, default=BUNDLE / 'patched/tools')
    parser.add_argument('--base-tools', type=Path, default=Path('tools'))
    args = parser.parse_args()
    if args.work.exists():
        print(f'usage: {args.work} exists', file=sys.stderr)
        return 2
    repo = Path.cwd().resolve()
    mirror = args.work / 'tools'
    mirror.mkdir(parents=True)
    for name, src in (('coherent_narration.py', args.patched_tools), ('narration_cache.py', args.base_tools)):
        (mirror / name).write_bytes((src / name).read_bytes())
    base = load(args.base_tools / 'coherent_narration.py', 'coherent_base')
    m = load(mirror / 'coherent_narration.py', 'coherent_patched')
    base.REPO = m.REPO = repo  # direction maps resolve against the real repository

    candidates, assemblies = {}, []
    for top in ROOTS:
        for dirpath, _, files in os.walk(top):
            for f in files:
                p = Path(dirpath) / f
                if f == 'assembly.json':
                    assemblies.append(p.parent)
                elif (f == 'script.tsv' or (f.endswith('.json') and 'units' in f)) and p.stat().st_size < 5_000_000:
                    candidates.setdefault(digest(p), p)
    pools = {'oliver': [], 'native': []}
    for out in sorted(assemblies):
        try:
            report = json.loads((out / 'report.json').read_text())
            manifest = json.loads((out / 'assembly.json').read_text())['manifest']
            script = candidates[report['meta']['script_sha256']]
            units = candidates[report['meta']['units_sha256']]
            plan = json.loads(units.read_text())
            labels = {json.loads(Path(u['generation_receipt']).read_text()).get('source') for u in manifest['units']}
        except (KeyError, OSError, ValueError, TypeError):
            continue
        kind = 'oliver' if labels == {'ElevenLabs UI, Oliver'} else 'native' if labels == {None} else None
        if 'contract-fixtures' in out.parts:
            continue  # deliberately broken probes refuse before any receipt is read
        cues = set(report['lines'])
        every = {c for u in plan['units'] for c in u['line_ids']}
        prefixes = {c.split('_')[0] for c in cues}
        if cues == every:
            chapter = None
        elif len(prefixes) == 1:
            chapter = prefixes.pop()
        else:
            continue
        if kind:
            size = sum(p.stat().st_size for p in out.glob('*.wav'))
            pools[kind].append((size, out, script, units, chapter))
    agree = differ = 0
    for kind, limit in (('oliver', args.oliver), ('native', args.native)):
        print(f'{kind}: {len(pools[kind])} retained assemblies with resolvable script and plan; sampling {min(limit, len(pools[kind]))} smallest')
        for size, out, script, units, chapter in sorted(pools[kind], key=lambda t: (t[0], str(t[1])))[:limit]:
            a, b = outcome(base, out, script, units, chapter), outcome(m, out, script, units, chapter)
            same = a == b
            agree += same
            differ += not same
            print(f"  {'SAME' if same else 'DIFF'} {out} (chapter {chapter or 'all'}, {size // 1024} KiB): "
                  f"canonical {a[:90]}" + ('' if same else f' | patched {b[:90]}'))
    print(f'{agree} agree, {differ} differ')
    return 0 if differ == 0 and agree > 0 else 1


if __name__ == '__main__':
    sys.exit(main())
