"""Real-input rehearsal of the v2 receipt against the retained Frederick files.

READ-ONLY on the repository; writes only into --work (a new scratch directory).
It is a schema-fit check, not intake or production: one take of one cue is
decoded with root's ffmpeg arguments so the patched branch can be exercised on
real bytes. The sentence ends it passes are PLACEHOLDERS (an even split), not
measured boundaries, and nothing here is listened to or transcribed.

Usage, from the repository root:
    python3 -I -B <bundle>/checks/frederick_rehearsal.py --work NEW_DIR [--patched-tools DIR]
Exit 0 all checks passed, 1 a check failed, 2 usage.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys

BUNDLE = Path(__file__).resolve().parents[1]
EC = Path('narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104')
TAKES = EC / 'frederick-stage05-01'
STAGE = EC / 'combined-repair-stage05/explainer_corpus/complex_analysis_visual'
VOICE = 'Frederick Surrey - Smooth and Velvety'
LABEL = 'ElevenLabs UI, Frederick Surrey'
CUE = '01_1'
FFMPEG_ARGS = ['-nostdin', '-n', '-v', 'error', '-ac', '1', '-ar', '24000', '-c:a', 'pcm_s16le']


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load(path, alias):
    spec = importlib.util.spec_from_file_location(alias, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--work', type=Path, required=True)
    parser.add_argument('--patched-tools', type=Path, default=BUNDLE / 'patched/tools',
                        help='directory holding the patched coherent_narration.py and narration_cache.py')
    parser.add_argument('--base-tools', type=Path, default=Path('tools'))
    args = parser.parse_args()
    if args.work.exists():
        print(f'usage: {args.work} exists', file=sys.stderr)
        return 2
    args.work.mkdir(parents=True)
    failures = []

    def check(name, ok, detail=''):
        print(('PASS ' if ok else 'FAIL ') + name + (f'  [{detail}]' if detail else ''))
        if not ok:
            failures.append(name)

    # The patched module needs narration_cache.py beside it; use a work mirror.
    mirror = args.work / 'tools'
    mirror.mkdir()
    for name, src in (('coherent_narration.py', args.patched_tools), ('narration_cache.py', args.base_tools)):
        (mirror / name).write_bytes((src / name).read_bytes())
    m = load(mirror / 'coherent_narration.py', 'coherent_patched')
    base = load(args.base_tools / 'coherent_narration.py', 'coherent_base')

    script_path, units_path = STAGE / 'script.tsv', STAGE / 'narration_units.json'
    script = m.read_script(script_path)
    plan = json.loads(units_path.read_text())
    units = {u['line_ids'][0]: u for u in plan['units'] if len(u['line_ids']) == 1}
    check('stage05 plan is one cue per unit and covers the script',
          len(units) == len(plan['units']) == len(script) and set(units) == set(script), f'{len(units)} units')

    # 1. Every retained request names its cue, exact current text and exact voice.
    bad_voice, bad_text, missing = [], [], []
    for cue, text in script.items():
        path = TAKES / f'{cue}-request01.json'
        if not path.exists():
            missing.append(cue)
            continue
        request = json.loads(path.read_text())
        if request.get('cue') != cue or request.get('voice') != VOICE:
            bad_voice.append(cue)
        if request.get('text') != text or request.get('text') != units[cue]['text']:
            bad_text.append(cue)
    check('every script cue has request01.json', not missing, ', '.join(missing) or f'{len(script)} requests')
    check('every request names its cue and the exact voice string', not bad_voice, ', '.join(bad_voice) or VOICE)
    check('every request text equals the stage05 script and unit text', not bad_text, ', '.join(bad_text) or 'exact')

    # 2. Retained takes listed by each downloads receipt still match their hashes.
    receipts = sorted(TAKES.glob('*-downloads01.json'))
    changed, takes = [], 0
    for receipt in receipts:
        body = json.loads(receipt.read_text())
        for take in body['takes']:
            takes += 1
            local = TAKES / Path(take['path']).name
            if not local.exists() or digest(local) != take['sha256']:
                changed.append(local.name)
    check('retained Frederick takes match their downloads receipts', not changed and takes > 0,
          ', '.join(changed) or f'{takes} takes in {len(receipts)} receipts')

    # 3. One real take through the patched v2 branch (placeholder sentence ends).
    source = TAKES / f'{CUE}-download01.wav'
    request = TAKES / f'{CUE}-request01.json'
    decoded = (args.work / f'{CUE}-download01-24k.wav').resolve()
    result = subprocess.run(['ffmpeg', *FFMPEG_ARGS[:4], '-i', str(source), *FFMPEG_ARGS[4:], str(decoded)],
                            capture_output=True, text=True, timeout=120)
    check('ffmpeg decode of one take (root arguments)', result.returncode == 0, result.stderr.strip()[:200])
    if result.returncode != 0:
        return 1
    info = m.cache.wav_identity(decoded)
    listed = next(t for t in json.loads((TAKES / f'{CUE}-downloads01.json').read_text())['takes']
                  if Path(t['path']).name == source.name)
    check('decoded frames = ceil(source frames / 2)', info['frames'] == (listed['frames'] + 1) // 2,
          f"{listed['frames']} -> {info['frames']}")
    row = {'cue': CUE, 'text': script[CUE], 'voice': VOICE, 'request': str(request), 'request_sha256': digest(request),
           'source': str(source), 'source_sha256': digest(source), 'wav': str(decoded), 'wav_sha256': info['sha256'],
           'samples': info['frames'], 'rate': info['sample_rate']}
    receipt = {'schema': m.EXTERNAL_UI_SCHEMA, 'source': LABEL, 'provider': 'ElevenLabs', 'voice': VOICE,
               'records': [row], 'provenance': 'REHEARSAL in scratch; placeholder sentence ends; not production.'}
    receipt_path = args.work / 'rehearsal-receipt.json'
    receipt_path.write_text(json.dumps(receipt, indent=1))
    parts = m.sentences(script[CUE])
    ends = [info['frames'] * (i + 1) // len(parts) for i in range(len(parts))]
    record = {'id': units[CUE]['id'], 'wav': str(decoded), 'sha256': info['sha256'],
              'generation_receipt': str(receipt_path), 'sentence_ends': ends, 'gap_after_samples': [0] * len(parts)}
    rows = m.unit_spans(units[CUE], script, record)
    rebuilt = b''.join(r[2] for r in rows)
    check('patched unit_spans accepts the real v2 record', len(rows) == len(parts), f'{len(parts)} sentences')
    check('spans cover every decoded sample in order', rebuilt == m.wav_pcm(decoded)[1], f"{info['frames']} frames")
    check('assembly_voice names Frederick exactly', m.assembly_voice([record]) == VOICE)
    try:
        base.unit_spans(units[CUE], script, record)
        check('canonical base refuses the same record', False, 'accepted')
    except KeyError as error:
        check('canonical base refuses the same record (the repaired defect)', True, f'KeyError {error}')
    for label, edit, pattern in (
            ('Oliver label on Frederick metadata', {'source': 'ElevenLabs UI, Oliver'}, 'provider or voice'),
            ('short voice name', {'voice': 'Frederick Surrey'}, 'provider or voice'),
            ('unversioned Frederick label', {'schema': None}, 'unrecognised external UI')):
        body = {k: v for k, v in dict(receipt, **edit).items() if v is not None}
        receipt_path.write_text(json.dumps(body))
        try:
            m.unit_spans(units[CUE], script, record)
            check(f'refuses {label}', False, 'accepted')
        except ValueError as error:
            check(f'refuses {label}', pattern in str(error), str(error)[:80])
    receipt_path.write_text(json.dumps(receipt, indent=1))
    print('ALL PASS' if not failures else f'{len(failures)} FAILED')
    return 0 if not failures else 1


if __name__ == '__main__':
    sys.exit(main())
