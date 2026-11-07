#!/usr/bin/env python3
"""Guard child: assemble two whole-take replacements, preserving the other 103 cues.

No generation, decode, signal processing, rendering or publication. The run
subcommand requires an admitted guard job and a successful frozen native build.
The audit-hooks subcommand is read-only and works on the local staged sources.
"""
from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import wave

ROOT = Path('/data/projects/jmev-stroke-probe-20261106')
Q = ROOT / 'qualification01/production-repair17'
TREE = ROOT / 'work/production-repair17/workspace'
JOB = ROOT / 'jobs/csd-production-assembly17'
PRIOR = ROOT / 'jobs/csd-continuous-rebind16/narration'
LEGACY = ROOT / 'work/production-repo07'
BUILD = ROOT / 'jobs/csd-production-repair-build17'
TOPIC = Path('explainer_corpus/complex_analysis_visual')
NEW_MAP = TOPIC / 'docs/narration-direction-map-frederick-production17.json'
REPLACED = {'08_5', '09_1'}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha(path):
    with Path(path).open('rb') as handle:
        return hashlib.file_digest(handle, 'sha256').hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def write_new(path, value):
    with Path(path).open('x') as handle:
        json.dump(value, handle, indent=2)
        handle.write('\n')


def script_rows(path):
    rows = {}
    for line in Path(path).read_text().splitlines():
        if not line.strip() or line.startswith('#'):
            continue
        cue, text = line.split('\t', 1)
        require(re.fullmatch(r'\d\d_\d+', cue) and cue not in rows and text, 'bad script cue')
        rows[cue] = text
    return rows


def terminal(path):
    receipt = read(path)
    require(receipt.get('exit') == 0 and receipt['inputs_sha256'] == receipt['final_inputs_sha256']
            and not receipt.get('resource_stop') and not receipt.get('stopped_for_resource_limit'),
            f'guarded prerequisite not stable: {path}')
    return receipt


def source_hooks(source_dir, texts, timing):
    """Audit actual literal and the two tuple-driven phrase hooks; never interpolate."""
    result = []
    for cue, filename in [('08_5', 'loop_sums.rs'), ('09_1', 'boundary.rs')]:
        source = (source_dir / filename).read_text()
        if cue == '08_5':
            body = source.split('fn goursat_proof(', 1)[1].split('pub struct LoopSums', 1)[0]
        else:
            body = source.split('let line = say(&kit, stage, "09_1")?;', 1)[1]
            body = body.split('let line = say(&kit, stage, "09_2")?;', 1)[0]
        phrases = re.findall(r'line\.(?:wait_for|until|at)\(\s*stage,\s*"([^"]+)"', body)
        if cue == '08_5':
            dynamic = re.findall(r'(?:false|true),\s*"([^"]+)",\s*r"', body)
            require(len(dynamic) == 2 and body.count('line.wait_for(stage, phrase)') == 1,
                    'unexpected dynamic Goursat phrase-hook shape')
            phrases.extend(dynamic)
        marks = {mark['byte']: mark for mark in timing['cues'][cue]['marks']}
        for phrase in dict.fromkeys(phrases):
            require(texts[cue].count(phrase) == 1, f'missing/ambiguous source phrase: {cue}: {phrase}')
            byte = len(texts[cue].split(phrase, 1)[0].encode())
            mark = marks.get(byte)
            result.append({'cue': cue, 'phrase': phrase, 'byte': byte,
                           'status': 'provisional_mark' if mark else 'unsupported',
                           'sample_24k': mark['sample_24k'] if mark else None,
                           'basis': mark.get('basis') if mark else 'no exact onset mark; no guess supplied'})
    return result


def relocate_map(original, tree, texts, script_hash):
    data = copy.deepcopy(original)
    require(len(data['visual_holds']) == 23 and len(data['helpers']) == 5, 'unexpected direction-map coverage')
    data['scripts']['complex_analysis_visual'] = script_hash
    for override in data['overrides']:
        if override['cue'] == '08_5':
            override['after'] = texts['08_5']
        require(override['after'] == texts[override['cue']], 'visual hold no longer follows the complete cue')
    audit = []
    for kind in ('visual_holds', 'helpers'):
        for row in data[kind]:
            relative = Path(row['path'])
            require(not relative.is_absolute() and '..' not in relative.parts, 'unsafe source evidence path')
            path = tree / relative
            lines = path.read_text().splitlines()
            indices = [i for i, line in enumerate(lines) if line.strip() == row['marker']]
            if kind == 'visual_holds':
                cue_pattern = re.compile(r'let line = say\(&kit, stage, "(\d\d_\d+)"\)\?;')
                sites = [(i, hit[1]) for i, line in enumerate(lines) if (hit := cue_pattern.search(line))]
                starts = [i for i, cue in sites if cue == row['cue']]
                require(len(starts) == 1, f'cue call not unique: {row["cue"]}')
                start = starts[0]
                end = next((i for i, _ in sites if i > start), len(lines))
                indices = [i for i in indices if start < i < end]
            require(len(indices) == 1, f'hold/helper marker not uniquely rebound: {row}')
            before = {'line': row['line'], 'sha256': row['sha256']}
            row.update(line=indices[0] + 1, sha256=sha(path))
            audit.append({'kind': kind, 'path': row['path'], 'cue': row.get('cue'),
                          'before': before, 'after': {'line': row['line'], 'sha256': row['sha256']},
                          'marker': row['marker']})
    data['scope'] = ('Production17 exact-source rebind: 23 native holds and five helpers. '
                     'Complete 08_5 text updated; durations unchanged. No listening or synchronization acceptance.')
    return data, audit


def run_command(job, name, argv):
    with (job / (name + '.stdout')).open('xb') as stdout, (job / (name + '.stderr')).open('xb') as stderr:
        completed = subprocess.run([str(arg) for arg in argv], cwd=LEGACY,
                                   stdout=stdout, stderr=stderr, timeout=1800, check=False)
    require(completed.returncode == 0, f'{name} failed; inspect retained stdout/stderr')
    return (job / (name + '.stdout')).read_text()


def run(args):
    admission = read(JOB / 'resource-receipt.json')
    require(admission.get('admitted') and 'exit' not in admission, 'assembly needs a live admitted guard job')
    terminal(BUILD / 'resource-receipt.json')
    terminal(ROOT / 'jobs/csd-continuous-rebind16/resource-receipt.json')
    qualification = read(BUILD / 'qualification.json')
    binary = ROOT / 'work/bin/production17'
    require(sha(binary) == qualification['binary_sha256'], 'built binary changed')
    original_files = read(BUILD / 'source-copy.json')['files']
    adapter = TREE / 'tools/coherent_narration.py'
    require(sha(adapter) == args.adapter_sha256 == sha(Q / 'coherent_narration.py'), 'wrong provider-aware adapter')
    adapter_backup = TREE / 'tools/coherent_narration.before-production17.py'
    require(sha(adapter_backup) == original_files['tools/coherent_narration.py'], 'missing exact prior adapter backup')
    for name, digest in original_files.items():
        if name != 'tools/coherent_narration.py':
            require(sha(TREE / name) == digest, f'build-frozen source changed before assembly: {name}')
    before_paths = {str(p.relative_to(TREE)) for p in TREE.rglob('*') if p.is_file()}
    require(before_paths == set(original_files) | {'tools/coherent_narration.before-production17.py'},
            'unexpected workspace additions before assembly')

    topic = TREE / TOPIC
    script = topic / 'script.tsv'
    texts = script_rows(script)
    old_plan = read(PRIOR / 'external-units.json')
    require(len(texts) == len(old_plan['units']) == 105, 'expected full 105-cue film')
    require([unit['line_ids'] for unit in old_plan['units']] == [[cue] for cue in texts], 'cue order changed')
    old_texts = {unit['line_ids'][0]: unit['text'] for unit in old_plan['units']}
    require({cue for cue in texts if texts[cue] != old_texts[cue]} == REPLACED, 'unexpected narration text changes')
    old_assembly, old_report = read(PRIOR / 'assembly.json'), read(PRIOR / 'report.json')
    require(old_assembly['report_sha256'] == sha(PRIOR / 'report.json'), 'prior report binding changed')
    require(old_assembly['manifest']['units_sha256'] == sha(PRIOR / 'external-units.json'), 'prior units binding changed')
    require(set(old_assembly['outputs']) == set(texts), 'prior output coverage differs')
    for cue in texts:
        require(sha(PRIOR / (cue + '.wav')) == old_assembly['outputs'][cue]['sha256'], 'prior WAV changed')
    timing_path = Q / 'timing-selected17.json'
    timing = read(timing_path)
    require(set(timing['cues']) == REPLACED, 'timing cue set differs')
    selected_receipt = Q / 'audio17/mapped-receipt.json'
    receipt = read(selected_receipt)
    require(receipt['schema'] == 'elevenlabs-ui-receipt-v2'
            and receipt['source'] == 'ElevenLabs UI, Frederick Surrey'
            and receipt['provider'] == 'ElevenLabs'
            and receipt['voice'] == 'Frederick Surrey - Smooth and Velvety', 'wrong selected provider/voice')
    records = {row['cue']: row for row in receipt['records']}
    require(len(receipt['records']) == 2 and set(records) == REPLACED, 'selected receipt coverage differs')
    for cue, row in records.items():
        marks = timing['cues'][cue]
        require(row['text'] == texts[cue] and row['voice'] == receipt['voice'], 'selected text/voice differs')
        require(marks['text_sha256'] == hashlib.sha256(texts[cue].encode()).hexdigest(), 'timing text binding differs')
        require(sha(row['wav']) == row['wav_sha256'] == marks['wav_sha256'], 'timing WAV binding differs')
        require(sha(row['source']) == row['source_sha256'] and sha(row['request']) == row['request_sha256'], 'raw/request changed')
        with wave.open(row['wav']) as wav:
            require(wav.getparams()[:3] == (1, 2, 24000) and wav.getnframes() == row['samples'], 'wrong complete decode format')
        ends, gaps = marks['sentence_ends'], marks['gap_after_samples']
        require(ends[-1] == row['samples'] and all(type(n) is int for n in ends + gaps), 'bad whole-take partition')
        require(all(b > a for a, b in zip([0] + ends, ends)) and not any(gaps), 'truncated take or inserted silence')
        require(len(ends) == len(gaps) == len(marks['sentence_starts']), 'sentence timing cardinality differs')
        require([m['sample_24k'] for m in marks['sentence_starts']] == [0] + ends[:-1], 'sentence onset partition differs')
        require(len({m['byte'] for m in marks['marks']}) == len(marks['marks']), 'duplicate word-onset byte')
        for mark in marks['marks']:
            require(0 <= mark['byte'] < len(texts[cue].encode()) and 0 <= mark['sample_24k'] < row['samples'], 'mark outside cue')
            require(abs(mark['seconds'] - mark['sample_24k'] / 24000) < 1e-9, 'mark seconds differ from sample')
            require(texts[cue].encode()[mark['byte']:].startswith(mark['phrase'].encode()), 'mark byte names another word')
    hooks = source_hooks(topic / 'src/chapters', texts, timing)
    write_new(JOB / 'phrase-hook-audit.json', hooks)
    require(all(row['status'] == 'provisional_mark' for row in hooks), 'unsupported phrase hooks; no guessed fallback permitted')

    old_map_path = TREE / old_plan['direction_contract']['map']['path']
    require(sha(old_map_path) == old_plan['direction_contract']['map']['sha256'], 'prior direction map changed')
    data, relocations = relocate_map(read(old_map_path), TREE, texts, sha(script))
    require(not (TREE / NEW_MAP).exists(), 'new direction map already exists')
    canonical_path = topic / 'narration_units.json'
    canonical_before = canonical_path.read_bytes()
    canonical = json.loads(canonical_before)
    require(canonical['direction_contract'] == old_plan['direction_contract'], 'canonical/external old contracts differ')
    backup = topic / 'narration_units.before-production17.json'
    require(not backup.exists() and not (JOB / 'narration').exists(), 'assembly outputs already exist')
    write_new(TREE / NEW_MAP, data)
    contract = copy.deepcopy(old_plan['direction_contract'])
    contract.update(script_sha256=sha(script), map={'path': str(NEW_MAP), 'sha256': sha(TREE / NEW_MAP)})
    external = copy.deepcopy(old_plan)
    for plan in (canonical, external):
        plan.update(script_sha256=sha(script), direction_contract=contract,
                    scope='Production17 complete Frederick Surrey UI takes; 103 retained cues, two full-cue replacements.')
        for unit in plan['units']:
            require(len(unit['line_ids']) == 1, 'canonical unit is not one cue')
            cue = unit['line_ids'][0]
            if cue in REPLACED:
                unit.update(text=texts[cue], external_ui_provider='elevenlabs')
    with backup.open('xb') as handle:
        handle.write(canonical_before)
    # The new workspace's canonical plan is an explicitly declared guard output.
    require(sha(canonical_path) == original_files[str(TOPIC / 'narration_units.json')], 'canonical changed at mutation boundary')
    canonical_path.write_text(json.dumps(canonical, indent=2) + '\n')
    units_path = JOB / 'external-units.json'
    write_new(units_path, external)
    manifest = copy.deepcopy(old_assembly['manifest'])
    manifest['units_sha256'] = sha(units_path)
    manifest['boundary_review'] = {
        'reviewer': 'Production17 retained-take assembly; provisional ASR timing',
        'listening': 'unreviewed',
        'basis': '103 prior complete cues retained; two complete Frederick Surrey decoded UI takes. '
                 'No samples removed, no inserted silence, no gain/time/pitch changes. '
                 'New sentence partitions and exact phrase hooks use provisional ASR marks, not heard onsets.'}
    for unit, row in zip(external['units'], manifest['units']):
        require(unit['id'] == row['id'], 'manifest unit ordering differs')
        cue = unit['line_ids'][0]
        if cue in REPLACED:
            selected, marks = records[cue], timing['cues'][cue]
            row.clear()
            row.update(id=unit['id'], wav=selected['wav'], sha256=selected['wav_sha256'],
                       generation_receipt=str(selected_receipt), sentence_ends=marks['sentence_ends'],
                       gap_after_samples=marks['gap_after_samples'])
    holds = {row['cue']: row for row in data['visual_holds']}
    require(len(manifest['direction_dispositions']) == 23, 'missing old direction dispositions')
    for disposition in manifest['direction_dispositions']:
        hold = holds[disposition['cue']]
        require(disposition['kind'] == 'native_visual_hold', 'unexpected disposition kind')
        disposition['evidence'] = {'path': str(TREE / hold['path']), 'sha256': hold['sha256']}
        disposition['reason'] = f'Exact native hold at {hold["path"]}:{hold["line"]}; complete cue retained, no inserted silence; listening unreviewed.'
    spans_path = JOB / 'spans.json'
    write_new(spans_path, manifest)
    out = JOB / 'narration'
    command = [sys.executable, '-I', '-B', adapter]
    run_command(JOB, 'assemble', command + ['assemble', '--script', script, '--units', units_path,
                '--spans', spans_path, '--out', out, '--require-directions'])
    with (out / 'external-units.json').open('xb') as handle:
        handle.write(units_path.read_bytes())
    assembled = read(out / 'assembly.json')
    report = read(out / 'report.json')
    old_marks = read(PRIOR / 'phrase-timings.json')
    phrase_marks = copy.deepcopy(old_marks)
    phrase_marks.update(script_sha256=sha(script), schema='complex-frederick-production17-v1',
                        source_sha256=sha(timing_path), prepare_receipt_sha256=sha(selected_receipt),
                        basis='103 retained timing entries; 08_5/09_1 provisional ASR word onsets, not heard boundaries')
    for cue in texts:
        if cue not in REPLACED:
            require(sha(out / (cue + '.wav')) == old_assembly['outputs'][cue]['sha256'], f'retained WAV changed: {cue}')
            require(report['lines'][cue] == old_report['lines'][cue], f'retained sentence marks changed: {cue}')
            require(phrase_marks['cues'][cue] == old_marks['cues'][cue], f'retained phrase marks changed: {cue}')
        else:
            phrase_marks['cues'][cue] = {'text_sha256': timing['cues'][cue]['text_sha256'],
                'wav_sha256': assembled['outputs'][cue]['sha256'], 'marks': timing['cues'][cue]['marks']}
    write_new(out / 'phrase-timings.json', phrase_marks)
    run_command(JOB, 'verify', command + ['verify', out, '--script', script,
                '--units', out / 'external-units.json', '--require-directions'])
    delivery_args = [sys.executable, '-I', '-B', TREE / 'tools/narration_delivery.py',
                     '--topic', topic, '--narration', out, '--consumer', 'render', '--gain-db', '0']
    for chapter in qualification['chapters']:
        delivery_args.extend(['--chapter-key', chapter])
    delivery = json.loads(run_command(JOB, 'delivery', delivery_args))
    require(delivery['verified_cues'] == delivery['selected_cues'] == 105 and delivery['plan_mode'] == 'external-ui', 'delivery not fully verified')
    allowed = {'tools/coherent_narration.py', str(TOPIC / 'narration_units.json')}
    for name, digest in original_files.items():
        if name not in allowed:
            require(sha(TREE / name) == digest, f'compiled source changed: {name}')
    final_files = {str(p.relative_to(TREE)): sha(p) for p in TREE.rglob('*') if p.is_file()}
    additions = {'tools/coherent_narration.before-production17.py', str(NEW_MAP),
                 str(TOPIC / 'narration_units.before-production17.json')}
    require(set(final_files) == set(original_files) | additions, 'unexpected final workspace additions')
    require(sha(backup) == original_files[str(TOPIC / 'narration_units.json')], 'canonical backup changed')
    write_new(JOB / 'workspace-after.json', {'files': final_files, 'changed_existing': sorted(allowed), 'added': sorted(additions)})
    # Guard stdout and resource receipts can still grow while this child runs.
    # Bind only our closed outputs here; the guard binds its own terminal files.
    closed_outputs = [JOB / name for name in (
        'phrase-hook-audit.json', 'external-units.json', 'spans.json', 'workspace-after.json',
        'assemble.stdout', 'assemble.stderr', 'verify.stdout', 'verify.stderr',
        'delivery.stdout', 'delivery.stderr')]
    closed_outputs.extend(p for p in out.rglob('*') if p.is_file())
    write_new(JOB / 'assembly17-summary.json', {
        'schema': 'production-assembly17-v1', 'narration': str(out),
        'script_sha256': sha(script), 'units_sha256': sha(out / 'external-units.json'),
        'canonical_sha256': sha(canonical_path), 'adapter_sha256': sha(adapter),
        'assembly_sha256': sha(out / 'assembly.json'), 'report_sha256': sha(out / 'report.json'),
        'phrase_timings_sha256': sha(out / 'phrase-timings.json'),
        'direction_map_sha256': sha(TREE / NEW_MAP), 'build_binary_sha256': sha(binary),
        'build_source_copy_sha256': sha(BUILD / 'source-copy.json'),
        'retained_wav_cues_byte_identical': 103, 'retained_sentence_and_phrase_cues': 103,
        'replaced_cues': sorted(REPLACED), 'sample_verified_cues': 105,
        'phrase_hook_audit': hooks, 'source_relocations': relocations, 'delivery': delivery,
        'output_hashes': {str(p.relative_to(JOB)): sha(p) for p in closed_outputs},
        'scope': 'Technical complete-take assembly and exact source binding only; '
                 'provisional ASR timing, no listening, fine synchronization, picture or release acceptance.'})
    print('Assembled and verified 105 cues: 103 byte-identical retained cues and two complete replacements.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest='command', required=True)
    execute = sub.add_parser('run')
    execute.add_argument('--adapter-sha256', required=True)
    audit = sub.add_parser('audit-hooks')
    audit.add_argument('--source-dir', type=Path, required=True)
    audit.add_argument('--script', type=Path, required=True)
    audit.add_argument('--timing', type=Path, required=True)
    args = parser.parse_args()
    if args.command == 'run':
        run(args)
    else:
        rows = source_hooks(args.source_dir, script_rows(args.script), read(args.timing))
        print(json.dumps({'hooks': rows, 'unsupported': [row for row in rows if row['status'] == 'unsupported']}, indent=2))
        return int(any(row['status'] == 'unsupported' for row in rows))
    return 0


if __name__ == '__main__':
    sys.dont_write_bytecode = True
    raise SystemExit(main())
