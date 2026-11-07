#!/usr/bin/env python3
"""Guarded 186-cue algebra assembly: replace only the complete Frederick 11_17.

The inspect subcommand reads current sources and ASR without writing. The run
subcommand writes only a fresh narration and receipts inside the admitted job.
No synthesis, decoding, source edits, picture rendering or listening verdict.
"""
from __future__ import annotations

import argparse
import copy
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import subprocess
import sys
import wave

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
REPO = HERE.parents[7]
B = REPO / 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
INPUTS = B / 'algebra-fixes-inputs02'
W = INPUTS / 'workspace'
C = W / 'explainer_corpus/abstract_algebra_expanded'
OLD = B / 'algebra-combined-repair-timeline01/narration'
JOB = B / 'algebra-fixes-assemble03'
DECODE = B / 'algebra-repair17-decode01/decode-receipt.json'
ASR = B / 'algebra-repair17-recognition01/11_17-take1.json'
BRIDGE = REPO / 'docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-frederick-stage07-reuse01/frederick_timing_bridge_stage07.py'
BRIDGE_SHA = 'b50479b62953fba7da2c2a8af0653030f926a5fcbacb5fccabe968aed8d0427a'
CUE = '11_17'


def require(ok, message):
    if not ok:
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


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def terminal(job):
    receipt = read(job / 'resource-receipt.json')
    require(receipt.get('admitted') and receipt.get('exit') == 0
            and receipt['inputs_sha256'] == receipt['final_inputs_sha256']
            and not any(receipt.get(k) for k in ('error', 'stop_reason', 'resource_stop', 'stopped_for_resource_limit')),
            f'prerequisite guard not terminal and stable: {job.name}')
    return receipt


def timing():
    """Use the existing pinned bridge's recognition support rules, without guesses."""
    require(Path.cwd().resolve() == REPO, 'run from the canonical repository root')
    require(sha(BRIDGE) == BRIDGE_SHA, 'pinned timing bridge changed')
    bridge = load(BRIDGE, 'algebra_assembly03_timing_bridge')
    bron = bridge.dep('log_preparation')
    decode = read(DECODE)
    require(decode['schema'] == 'elevenlabs-ui-receipt-v2'
            and decode['source'] == 'ElevenLabs UI, Frederick Surrey'
            and decode['provider'] == 'ElevenLabs'
            and decode['voice'] == 'Frederick Surrey - Smooth and Velvety', 'wrong selected narrator receipt')
    require(len(decode['records']) == 1 and decode['records'][0]['cue'] == CUE, 'expected exactly one selected cue')
    record = decode['records'][0]
    require(record['voice'] == decode['voice'] and record['rate'] == 24000, 'wrong selected voice/rate')
    for name, key in [('wav', 'wav_sha256'), ('source', 'source_sha256'), ('request', 'request_sha256')]:
        require(sha(record[name]) == record[key], f'selected {name} changed')
    with wave.open(record['wav']) as wav:
        require(wav.getparams()[:3] == (1, 2, 24000) and wav.getnframes() == record['samples'], 'wrong full decoded WAV')
    script = dict(line.split('\t', 1) for line in (C / 'script.tsv').read_text().splitlines()
                  if line and not line.startswith('#'))
    text = script[CUE]
    require(text == record['text'] == (HERE / '11_17-frederick.txt').read_text().strip(), 'selected text differs')
    request = read(record['request'])
    require(request['cue'] == CUE and request['text'] == text and request['voice'] == record['voice'], 'request differs')
    recognition = read(ASR)
    require(Path(recognition['input_path']).resolve() == Path(record['source']).resolve(), 'ASR is not the selected take')
    require(recognition['result']['backend'] == 'whisper_cpp', 'unexpected recognition backend')
    segments = recognition['result']['segments']
    duration = record['samples'] / 24000
    starts, _, differences = bridge.take_points(bron, CUE, 'selected-take1', text, segments, duration, [])
    alignment = bridge.align(bron, text, segments)
    marks, unsupported = [], []
    for index, (word, char) in enumerate(alignment[0]):
        point = bridge.place(index, alignment, duration, -1.0)
        if point['status'] != 'supported':
            unsupported.append({'char': char, 'word': word, **point})
            continue
        marks.append({'byte': len(text[:char].encode()), 'seconds': point['sample_24k'] / 24000,
                      'sample_24k': point['sample_24k'], 'phrase': text[char:].split()[0],
                      'window_s': point['window_s'], 'abuts_previous_word': point.get('abuts_previous_word', False),
                      'basis': 'provisional recognized word-segment start; not a heard onset'})
    source = (C / 'src/permutations.rs').read_text()
    body = source.split('fn overlap_warning(', 1)[1].split('// ---- 11_18', 1)[0]
    hooks = [int(n) for n in re.findall(r'k\.until_sentence\(stage, "11_17", (\d+)\)', body)]
    require(hooks == [1, 2, 3, 4] and len(starts) == 5, 'native sentence-hook shape changed')
    require('until_phrase(' not in body and 'phrase_time(' not in body, 'new phrase hook needs an explicit audit')
    unsupported_hooks = [{'sentence': index, 'status': starts[index]['status']} for index in hooks
                         if starts[index]['status'] != 'supported']
    ends = [row.get('sample_24k') for row in starts[1:]] + [record['samples']]
    if not unsupported_hooks:
        require(all(type(x) is int for x in ends) and all(b > a for a, b in zip([0] + ends, ends)), 'invalid sample partition')
    require(len({row['byte'] for row in marks}) == len(marks), 'duplicate word mark')
    require(all(a['sample_24k'] <= b['sample_24k'] for a, b in zip(marks, marks[1:])), 'word marks out of order')
    return {'cue': CUE, 'text_sha256': hashlib.sha256(text.encode()).hexdigest(),
            'source_wav_sha256': record['wav_sha256'], 'samples': record['samples'],
            'sentence_starts': starts, 'sentence_ends': ends, 'gap_after_samples': [0] * len(ends),
            'native_sentence_hooks': hooks, 'unsupported_hooks': unsupported_hooks,
            'marks': marks, 'unsupported_word_marks': unsupported, 'differences': differences,
            'recognition': {'path': str(ASR), 'sha256': sha(ASR), 'input_sha256': sha(record['source']),
                            'model': recognition['request']['model']},
            'bridge_sha256': BRIDGE_SHA, 'tokenizer': bridge.DEPENDENCIES['log_preparation'],
            'claim': 'Full-take sample partition and provisional recognition marks only; no added silence, listening or fine-sync acceptance.'}, record


def run_command(name, argv):
    with (JOB / (name + '.stdout')).open('xb') as out, (JOB / (name + '.stderr')).open('xb') as err:
        result = subprocess.run([str(x) for x in argv], cwd=W, stdout=out, stderr=err, timeout=1800, check=False)
    require(result.returncode == 0, f'{name} failed; retained stdout/stderr contain evidence')
    return (JOB / (name + '.stdout')).read_text()


def run():
    admission = read(JOB / 'resource-receipt.json')
    require(admission.get('admitted') and 'exit' not in admission, 'requires a live admitted assembly guard')
    for name in ('algebra-fixes-prepare02', 'algebra-fixes-build02', 'algebra-repair17-decode01',
                 'algebra-repair17-recognition01', 'algebra-combined-repair-timeline01', 'algebra-corner-contrast-timeline01'):
        terminal(B / name)
    require(not (JOB / 'narration').exists(), 'output already exists')
    frozen = read(INPUTS / 'build02-inputs.json')['files']
    require(len({row['path'] for row in frozen}) == len(frozen), 'duplicate build input')
    for row in frozen:
        require(sha(row['path']) == row['sha256'], f'build input changed: {row["path"]}')
    source_before = {str(p.relative_to(W)): sha(p) for p in W.rglob('*') if p.is_file()}
    require((INPUTS / 'abstract_algebra_expanded').is_file(), 'missing successful native build artifact')
    chapters = (INPUTS / 'native-chapters01.txt').read_text().splitlines()
    require(len(chapters) == len(set(chapters)) == 16, 'wrong native chapter coverage')
    selected, record = timing()
    write_new(JOB / 'timing03.json', selected)
    require(not selected['unsupported_hooks'], 'required sentence onset unsupported; no estimated fallback')
    require(not selected['differences'], 'selected recognition differs from exact text; needs review')
    adapter = W / 'tools/coherent_narration.py'
    tool = load(adapter, 'algebra_assembly03_adapter')
    plan_path = C / 'narration_units.json'
    script, units = tool.read_units(C / 'script.tsv', plan_path, require_directions=True)
    require(len(units) == len(script) == 186, 'expected complete 186-cue algebra film')
    plan = read(plan_path)
    old_plan = read(OLD / 'external-units.json')
    require([unit['line_ids'] for unit in units] == [[cue] for cue in script], 'new unit order differs from script')
    require([unit['id'] for unit in units] == [unit['id'] for unit in old_plan['units']], 'stable unit IDs changed')
    require([u['line_ids'][0] for u, old in zip(units, old_plan['units']) if u != old] == [CUE], 'more than 11_17 unit changed')
    mapping = W / plan['direction_contract']['map']['path']
    require(sha(mapping) == plan['direction_contract']['map']['sha256'], 'stage14 direction map changed')
    mapdata = read(mapping)
    require(len(mapdata['visual_holds']) == len(mapdata['overrides']) == 16, 'direction coverage changed')
    for hold in mapdata['visual_holds']:
        path = W / hold['path']
        require(sha(path) == hold['sha256'] and path.read_text().splitlines()[hold['line'] - 1].strip() == hold['marker'],
                f'stage14 native hold evidence differs: {hold["cue"]}')
    old_assembly, old_report = read(OLD / 'assembly.json'), read(OLD / 'report.json')
    require(old_assembly['report_sha256'] == sha(OLD / 'report.json'), 'old report hash differs')
    require(old_assembly['manifest']['units_sha256'] == sha(OLD / 'external-units.json'), 'old plan hash differs')
    require(set(old_assembly['outputs']) == set(script), 'old assembly cue coverage differs')
    for cue in script:
        require(sha(OLD / (cue + '.wav')) == old_assembly['outputs'][cue]['sha256'], f'old WAV changed: {cue}')
    manifest = copy.deepcopy(old_assembly['manifest'])
    require([row['id'] for row in manifest['units']] == [unit['id'] for unit in units], 'manifest units differ')
    require(len(manifest['direction_dispositions']) == 16 and all(row['kind'] == 'retained_source'
            for row in manifest['direction_dispositions']), 'unexpected source dispositions')
    require(CUE not in {row['cue'] for row in mapdata['overrides']}, '11_17 now has a direction requiring explicit disposition')
    # The source dispositions refer to unchanged retained take evidence; keep them exactly.
    target = next(row for row in manifest['units'] if row['id'] == next(unit['id'] for unit in units if unit['line_ids'] == [CUE]))
    target.clear()
    target.update(id=next(unit['id'] for unit in units if unit['line_ids'] == [CUE]),
                  wav=record['wav'], sha256=record['wav_sha256'], generation_receipt=str(DECODE),
                  sentence_ends=selected['sentence_ends'], gap_after_samples=selected['gap_after_samples'])
    manifest['units_sha256'] = sha(plan_path)
    manifest['boundary_review'] = {
        'reviewer': 'Production repair17 sample-preserving assembly', 'listening': 'unreviewed',
        'basis': '185 prior cues unchanged. Selected complete Frederick Surrey 11_17 take partitioned at '
                 'four supported provisional ASR sentence onsets; no removed samples, inserted silence, gain/time/pitch processing.',
        'audit': str(JOB / 'timing03.json'), 'audit_sha256': sha(JOB / 'timing03.json')}
    spans = JOB / 'spans03.json'
    write_new(spans, manifest)
    out = JOB / 'narration'
    base = [sys.executable, '-I', '-B', adapter]
    run_command('assemble', base + ['assemble', '--script', C / 'script.tsv', '--units', plan_path,
                '--spans', spans, '--require-directions', '--out', out])
    with (out / 'external-units.json').open('xb') as handle:
        handle.write(plan_path.read_bytes())
    assembled, report = read(out / 'assembly.json'), read(out / 'report.json')
    previous_marks = read(OLD / 'phrase-timings.json')
    require(set(previous_marks['cues']) == set(script), 'prior phrase coverage differs')
    marks = copy.deepcopy(previous_marks)
    marks.update(script_sha256=sha(C / 'script.tsv'),
                 basis='185 cue timing entries retained; 11_17 provisional ASR word onsets from selected complete take1.',
                 listening_accepted=False, fine_sync_accepted=False, independent_av_accepted=False)
    marks['cues'][CUE] = {'source_wav_sha256': record['wav_sha256'],
        'wav_sha256': assembled['outputs'][CUE]['sha256'], 'text_sha256': selected['text_sha256'],
        'recognition': selected['recognition'], 'marks': selected['marks'],
        'unsupported_word_marks': selected['unsupported_word_marks']}
    for cue in script:
        if cue == CUE:
            continue
        require(sha(out / (cue + '.wav')) == old_assembly['outputs'][cue]['sha256'], f'retained WAV changed: {cue}')
        require(report['lines'][cue] == old_report['lines'][cue], f'retained sentence timing changed: {cue}')
        require(marks['cues'][cue] == previous_marks['cues'][cue], f'retained phrase timing changed: {cue}')
    with wave.open(record['wav']) as source, wave.open(str(out / (CUE + '.wav'))) as target_wav:
        require(source.getparams() == target_wav.getparams(), 'replacement WAV parameters changed')
        require(source.readframes(source.getnframes()) == target_wav.readframes(target_wav.getnframes()), 'replacement PCM changed')
    write_new(out / 'phrase-timings.json', marks)
    run_command('verify', base + ['verify', out, '--script', C / 'script.tsv',
                '--units', out / 'external-units.json', '--require-directions'])
    delivery_args = [sys.executable, '-I', '-B', W / 'tools/narration_delivery.py',
                     '--topic', C, '--narration', out, '--consumer', 'render', '--gain-db', '0']
    for chapter in chapters:
        delivery_args += ['--chapter-key', chapter]
    delivery = json.loads(run_command('delivery', delivery_args))
    require(delivery['verified_cues'] == delivery['selected_cues'] == 186 and delivery['plan_mode'] == 'external-ui', 'incomplete native delivery verification')
    require({str(p.relative_to(W)): sha(p) for p in W.rglob('*') if p.is_file()} == source_before, 'workspace mutated')
    for row in frozen:
        require(sha(row['path']) == row['sha256'], f'build input changed during assembly: {row["path"]}')
    closed = [JOB / name for name in ('timing03.json', 'spans03.json', 'assemble.stdout', 'assemble.stderr',
                                     'verify.stdout', 'verify.stderr', 'delivery.stdout', 'delivery.stderr')]
    closed += [p for p in out.rglob('*') if p.is_file()]
    write_new(JOB / 'assembly03-summary.json', {
        'schema': 'algebra-fixes-assembly03-v1', 'narration': str(out), 'workspace': str(W),
        'script_sha256': sha(C / 'script.tsv'), 'units_sha256': sha(out / 'external-units.json'),
        'adapter_sha256': sha(adapter), 'assembly_sha256': sha(out / 'assembly.json'),
        'report_sha256': sha(out / 'report.json'), 'phrase_timings_sha256': sha(out / 'phrase-timings.json'),
        'direction_map_sha256': sha(mapping), 'binary_sha256': sha(INPUTS / 'abstract_algebra_expanded'),
        'build_inputs_sha256': sha(INPUTS / 'build02-inputs.json'), 'workspace_files': source_before,
        'retained_wav_cues_byte_identical': 185, 'retained_sentence_and_phrase_cues': 185,
        'replaced_cues': [CUE], 'sample_verified_cues': 186, 'replacement_pcm_byte_identical': True,
        'native_sentence_hooks': selected['native_sentence_hooks'], 'unsupported_hooks': selected['unsupported_hooks'],
        'unsupported_word_marks': selected['unsupported_word_marks'], 'delivery': delivery,
        'output_hashes': {str(p.relative_to(JOB)): sha(p) for p in closed},
        'scope': 'Full sample reconstruction and source-bound provisional timing only; no listening, fine-sync, picture, score or release acceptance.'})
    print('Assembled and verified 186 algebra cues; 185 WAVs/timing entries unchanged, one complete 11_17 replacement.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=('inspect', 'run'))
    args = parser.parse_args()
    if args.command == 'run':
        run()
        return 0
    audit, _ = timing()
    print(json.dumps(audit, indent=2))
    return int(bool(audit['unsupported_hooks']))


if __name__ == '__main__':
    raise SystemExit(main())
