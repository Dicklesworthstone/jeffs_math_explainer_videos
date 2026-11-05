#!/usr/bin/env python3
"""Rehearse the four controllers end to end in a temporary folder.

Real: the controllers, each controller's own wait and launch code, the script and narration (read
only), ffmpeg and ffprobe on a tiny movie, PIL, the start scripts' shell, and one read-only --check.
Fake: the guard, the binary and the job tree. No real guard, render, score or thumbnail runs, and
nothing is written in the repository. Run from the repository root:

    python3 -I -B <bundle>/checks/rehearse_downstream.py
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
STAGES = ('score', 'preview', 'pictures', 'thumbnail')
RUN = {'score': 'run_score', 'preview': 'run_preview', 'pictures': 'extract', 'thumbnail': 'render'}
DONE = {'score': 'score-summary.json', 'preview': 'preview-summary.json',
        'pictures': 'pictures/extraction.json', 'thumbnail': 'output/thumbnail-receipt.json'}
FRAMES = 6
# case: (stage that must refuse, words the refusal must contain)
TIMELINE_CASES = {
    'failed-timeline': ('score', "assert d.get('admitted') is True and d.get('exit') == 0, job"),
    'unfinished-timeline': ('score', 'Timeline is not successfully complete'),
    'guard-policy-timeline': ('score', 'Guard policy changed'),
    'debug-binary': ('score', 'Timeline binary differs'),
    'baseline-claim': ('score', 'Full baseline equivalence cannot be claimed'),
    'prior-differs': ('score', 'Prior table differs'),
    'table-edited': ('score', 'New table changed'),
    'missing-chapter': ('score', "assert {p.stem.removesuffix('.timeline')"),
    'missing-cue-log': ('score', 'for chapter in T.CHAPTERS for table in T.TABLES'),
    'cue-order': ('score', 'assert seen == ordered'),
    'cue-count': ('score', 'assert seen == ordered'),
    'frozen-input': ('score', "assert sha(Path(row['path'])) == row['sha256']"),
    'duplicate-job': ('score', 'Duplicate score refused'),
}
RUN_CASES = {
    'guard-policy-score': ('score', 'Guard policy changed'),
    'existing-score': ('score', 'Preserve existing score outputs'),
    'score-exit': ('score', 'CalledProcessError'),
    'qa-fail': ('score', "assert report['chapter'] == chapter and report['passed'] is True"),
    'qa-metric': ('score', "assert report['metrics'] and all(m.get('pass') is True"),
    'qa-missing': ('score', "assert {p.stem for p in qa_folder.glob('*.json')} == set(T.CHAPTERS)"),
    'qa-edited': ('preview', 'assert S.sha(path) == digest'),
    'no-score-receipt': ('preview', 'assert T.success(S.JOB)'),
    'existing-preview': ('preview', 'Existing preview must be preserved'),
    'render-exit': ('preview', 'CalledProcessError'),
    'timeline-differs': ('preview', 'Preview table differs: 06_'),
    'cues-differs': ('preview', 'Preview table differs: 06_'),
    'cuelog-row': ('preview', 'Preview cue log differs: 06_'),
    'cuelog-foreign': ('preview', 'Preview score rows differ: 06_'),
    'cuelog-noscore': ('preview', 'Preview score rows differ: 06_'),
    'cuelog-wrong-chapter': ('preview', 'Preview score rows differ: 06_'),
    **{f'cuelog-score-{name}': ('preview', 'Preview score rows differ: 06_') for name in (
        'hash', 'bytes', 'frame', 'fps', 'offset', 'offsetbits', 'gain', 'gainbits', 'rate', 'channels', 'samples', 'start')},
    'score-parts-gap': ('preview', 'Preview score rows differ: 02_'),
    'score-parts-short': ('preview', 'Preview score rows differ: 02_'),
    'lint-missing': ('preview', 'Rendered lint report missing: 10_'),
    'mp4-missing': ('preview', "assert {p.stem for p in OUT.glob('*.mp4')} == set(T.CHAPTERS)"),
    'frames-short': ('preview', "assert int(v['nb_frames']) == frames"),
    'no-audio': ('preview', 'assert len(video) == len(audio) == 1'),
    'preview-file-changed': ('preview', "assert S.sha(Path(row['path'])) == row['sha256']"),
    'lint-removed': ('pictures', 'Required rendered layout report missing'),
    'no-preview-receipt': ('pictures', 'assert T.success(P.JOB)'),
    'thumb-size': ('thumbnail', "assert im.format == 'PNG' and im.size == (3840, 2160)"),
}
FAKE = r'''#!PYTHON -I
import hashlib, json, shutil, struct, sys
from pathlib import Path
here = Path(sys.argv[0]).resolve().parent
c = json.loads((here / 'config.json').read_text())
case, args = c['case'], sys.argv[1:]
FIELDS = {'cuelog-score-hash': (1, '0' * 64), 'cuelog-score-bytes': (2, '9'), 'cuelog-score-frame': (3, '7'),
          'cuelog-score-fps': (4, '30'), 'cuelog-score-offsetbits': (6, '0x0'), 'cuelog-score-gain': (7, '-6.0'),
          'cuelog-score-gainbits': (8, '0x1'), 'cuelog-score-rate': (9, '44100'), 'cuelog-score-channels': (10, '1'),
          'cuelog-score-samples': (11, '4799'), 'cuelog-score-start': (12, '1')}
out = Path(args[args.index('--out') + 1])
if args[0] == 'score':
    assert args[:5] == ['score', '--narration', c['narration'], '--timelines', c['timeline_out']]
    if case == 'score-exit':
        sys.exit(3)
    (out / 'stems/qa').mkdir(parents=True)
    for n, chapter in enumerate(c['chapters']):
        stem = out / 'stems' / f'{chapter}.music.f32.wav'
        if n == 1:
            names = [f'{stem.name}.part000{k}.wav' for k in (0, 1)]
            for name in names:
                (stem.parent / name).write_text(name)
            stem.with_suffix('.parts.tsv').write_text('score-parts\t1\n' + ''.join(
                f"{k * (2401 if case == 'score-parts-gap' else 2400)}\t{2399 if k and case == 'score-parts-short' else 2400}\t48000\tx\t{name}\n"
                for k, name in enumerate(names)))
        else:
            stem.write_text(chapter)
        if case == 'qa-missing' and n == 14:
            continue
        (stem.parent / 'qa' / f'{chapter}.json').write_text(json.dumps({
            'chapter': chapter, 'passed': not (case == 'qa-fail' and n == 3),
            'metrics': [{'name': 'level', 'pass': not (case == 'qa-metric' and n == 7)}, {'name': 'note', 'pass': None}]}))
elif args[0] == 'render':
    stems = Path(args[args.index('--score') + 1])
    assert args[:7] == ['render', 'all', '--res', '960x540', '--fps', '60', '--narration']
    assert args[-5:] == ['--lint', '--crf', '18', '--preset', 'veryfast']
    if case == 'render-exit':
        sys.exit(3)
    out.mkdir()
    timeline = Path(c['timeline_out'])
    for n, chapter in enumerate(c['chapters']):
        hit = n == 5
        for table in ('timeline', 'cues'):
            text = (timeline / f'{chapter}.{table}.tsv').read_text()
            (out / f'{chapter}.{table}.tsv').write_text(text + ('breath\t1\t1\n' if hit and case == table + '-differs' else ''))
        base = (timeline / f'{chapter}.cuelog.tsv').read_text()
        stem = stems / f"{c['chapters'][0] if hit and case == 'cuelog-wrong-chapter' else chapter}.music.f32.wav"
        parts = stem.with_suffix('.parts.tsv')
        files = [(stem, 0, c['frames'] * 800)]
        if parts.exists():
            fields = [line.split('\t') for line in parts.read_text().splitlines()[1:]]
            files = [(stem.with_name(f[4]), int(f[0]), int(f[1])) for f in fields]
        laid = ''
        for path, start, samples in files:
            offset = start / 48000 - c['frames'] / 60 + (1 / 48000 if hit and case == 'cuelog-score-offset' else 0)
            exact = struct.unpack('>Q', struct.pack('>d', offset))[0]
            row = [str(path), hashlib.sha256(path.read_bytes()).hexdigest(), str(path.stat().st_size), str(c['frames']), '60',
                   repr(offset), f'{exact:#018x}', '0.0', '0x0000000000000000', '48000', '2', str(samples), str(start)]
            if hit and case in FIELDS:
                index, value = FIELDS[case]
                row[index] = value
            laid += '\t'.join(row) + '\n'
        if hit and case == 'cuelog-row':
            base = base.replace('\t60\t', '\t30\t', 1)
        if hit and case == 'cuelog-noscore':
            laid = ''
        if hit and case == 'cuelog-foreign':
            laid += laid.replace('music.f32.wav', 'extra.wav')
        (out / f'{chapter}.cuelog.tsv').write_text(base + laid)
        if not (case == 'lint-missing' and n == 9):
            (out / f'{chapter}.layout.tsv').write_text('' if n else '01_1 +0.0s @0.04s\ttight\t"z" stands 0.20 from the edge\n')
        name = {'frames-short': 'tiny5.mp4', 'no-audio': 'silent.mp4'}.get(case, 'tiny.mp4') if n == 2 else 'tiny.mp4'
        if not (case == 'mp4-missing' and n == 2):
            shutil.copy(here / name, out / f'{chapter}.mp4')
elif args[0] == 'thumbnail':
    assert args[1:3] == ['--res', '3840x2160']
    out.mkdir()
    shutil.copy(here / ('small.png' if case == 'thumb-size' else 'thumb.png'), out / 'complex_analysis_visual_thumbnail.png')
'''
results = []


def check(name, ok, detail):
    results.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'} {name}: {detail}", flush=True)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load(stage):
    spec = importlib.util.spec_from_file_location('controller_' + stage, BUNDLE / f'{stage}_optimized01.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def layers(stage, H):
    P = H if stage == 'preview' else getattr(H, 'P', None)
    return (H if stage == 'score' else P.S), P


def patched(stage, tmp):
    """Point the whole chain of loaded controllers at the fixture, never at the real tree."""
    H = load(stage)
    S, P = layers(stage, H)
    T = S.T
    B = tmp / 'B'
    T.B = T.T.B = B
    T.BIN, T.MANIFEST, T.PRIOR_OUT = tmp / 'fake', tmp / 'timeline-inputs.json', tmp / 'prior'
    T.STAGE, T.GUARD = tmp / 'timeline-stage', tmp / 'fake-guard'
    T.BIN_SHA256 = sha(tmp / 'fake')
    T.guard_policy = lambda: None
    S.STAGE = tmp / 'downstream'
    S.TIMELINE_EXIT = T.STAGE / 'controller01.exit'
    S.EXIT = S.STAGE / 'optimized-score-controller01.exit'
    S.MANIFEST, S.OUT = S.STAGE / 'optimized-score-inputs01.json', B / S.JOB / 'scored'
    if P:
        P.EXIT = S.STAGE / 'optimized-preview-controller01.exit'
        P.MANIFEST, P.OUT = S.STAGE / 'optimized-preview-inputs01.json', B / P.JOB / 'preview'
    if stage == 'pictures':
        H.MANIFEST, H.OUT = S.STAGE / 'optimized-picture-inputs01.json', B / H.JOB / 'pictures'
    if stage == 'thumbnail':
        H.MANIFEST, H.OUT = S.STAGE / 'optimized-thumbnail-inputs01.json', B / H.JOB / 'output'
    return H


def write_json(path, data):
    path.write_text(json.dumps(data, indent=1))


def success_receipt(T, command=()):
    return {'admitted': True, 'exit': 0, 'finished': 'fixture', 'inputs_sha256': 'a', 'final_inputs_sha256': 'a',
            'policy': T.POLICY, 'command': list(command)}


def movies(folder):
    """One real small movie, and two that must be refused."""
    color = ['-f', 'lavfi', '-i', 'color=c=black:s=960x540:r=60']
    sound = ['-f', 'lavfi', '-i', 'anullsrc=r=48000:cl=stereo']
    for name, count, inputs in [('tiny.mp4', FRAMES, color + sound), ('tiny5.mp4', FRAMES - 1, color + sound),
                                ('silent.mp4', FRAMES, color)]:
        subprocess.run(['ffmpeg', '-v', 'error', '-nostdin', *inputs, '-frames:v', str(count), '-t', '0.1',
                        '-c:v', 'libx264', '-preset', 'ultrafast', '-pix_fmt', 'yuv420p', str(folder / name)], check=True)
    from PIL import Image
    Image.new('RGB', (3840, 2160)).save(folder / 'thumb.png')
    Image.new('RGB', (64, 36)).save(folder / 'small.png')


def fixture(case, tmp, shared):
    """A finished timeline job with 15 chapters and 105 cues, as the recovery leaves it."""
    tmp.mkdir()
    for name in ('tiny.mp4', 'tiny5.mp4', 'silent.mp4', 'thumb.png', 'small.png'):
        shutil.copy(shared / name, tmp / name)
    S = load('score')
    T = S.T
    (tmp / 'fake').write_text(FAKE.replace('PYTHON', sys.executable))
    (tmp / 'fake').chmod(0o755)
    (tmp / 'fake-guard').write_text(f"import subprocess, sys\nsys.exit(subprocess.run([sys.executable, '-I', '-B', {str(HERE)!r}, '--guard', {str(tmp)!r}, *sys.argv[1:]]).returncode)\n")
    job = tmp / 'B' / T.JOB
    out = job / 'timeline-output'
    out.mkdir(parents=True)
    (tmp / 'B' / T.BUILD_JOB).mkdir()
    write_json(tmp / 'B' / T.BUILD_JOB / 'resource-receipt.json', success_receipt(T))
    (tmp / 'prior').mkdir()
    (tmp / 'timeline-stage').mkdir()
    (tmp / 'downstream').mkdir()
    script = T.script_ids()
    chapters = []
    for chapter in T.CHAPTERS:
        ids = [cue for cue in script if cue.startswith(chapter[:2] + '_')]
        rows = ['# explainer timeline v1', f'chapter\t{chapter}', 'fps\t60', f'frames\t{FRAMES}']
        rows += [f"line\t{cue}\t{min(k, FRAMES - 1)}\t0\t{T.N / (cue + '.wav')}" for k, cue in enumerate(ids)]
        rows.append('breath\t3\t0.05')
        (out / f'{chapter}.timeline.tsv').write_text('\n'.join(rows) + '\n')
        (out / f'{chapter}.cues.tsv').write_text(''.join(f'{cue}\t0.000\t1.000\n' for cue in ids))
        (out / f'{chapter}.cuelog.tsv').write_text('# path\tsha256\n' + ''.join(
            f"{T.N / (cue + '.wav')}\tx\t8\t{min(k, FRAMES - 1)}\t60\t0.0\t0x0\t-\t-\t24000\t1\t1\t0\n" for k, cue in enumerate(ids)))
        chapters.append({'chapter': chapter, 'frames': FRAMES, 'cues': ids, 'timeline_sha256': sha(out / f'{chapter}.timeline.tsv')})
    tables = []
    for chapter in T.PRIOR_CHAPTERS:
        for table in T.TABLES:
            name = f'{chapter}.{table}.tsv'
            shutil.copy(out / name, tmp / 'prior' / name)
            tables.append({'table': name, 'lines': 1, 'normalized_equal': True, 'prior_sha256': sha(out / name), 'new_sha256': sha(out / name)})
    summary = {'chapters': chapters, 'cues': 105, 'fps': 60, 'native_oracle_fact_rows_matched': 131,
               'total_frames': FRAMES * 15, 'binary_sha256': sha(tmp / 'fake'), 'profile': 'release',
               'failed_job': T.T.JOB, 'prior_chapters_compared': list(T.PRIOR_CHAPTERS), 'prior_tables_compared': 12,
               'chapters_without_prior_table': list(T.CHAPTERS[4:]), 'full_baseline_equivalence_established': False}
    compared = {'prior_job': T.T.JOB, 'new_binary_sha256': sha(tmp / 'fake'), 'tables': tables}
    receipt = success_receipt(T)
    if case == 'failed-timeline':
        receipt['exit'] = 1
    if case == 'unfinished-timeline':
        del receipt['finished']
    if case == 'guard-policy-timeline':
        receipt['policy'] = dict(T.POLICY, work_cap_gib=48)
    if case == 'debug-binary':
        summary['binary_sha256'] = T.FAILED_BIN_SHA256
    if case == 'baseline-claim':
        summary['full_baseline_equivalence_established'] = True
    if case == 'prior-differs':
        tables[3]['normalized_equal'] = False
    if case == 'table-edited':
        with (out / tables[0]['table']).open('a') as handle:
            handle.write('\n')
    if case == 'missing-chapter':
        (out / f'{T.CHAPTERS[14]}.timeline.tsv').rename(tmp / 'moved.tsv')
    if case == 'missing-cue-log':
        (out / f'{T.CHAPTERS[14]}.cuelog.tsv').rename(tmp / 'moved.tsv')
    if case in ('cue-order', 'cue-count'):
        item = chapters[14]
        path = out / f"{item['chapter']}.timeline.tsv"
        ids = item['cues']
        lines = path.read_text().splitlines()
        wanted = ids[:-1] if case == 'cue-count' else [ids[1], ids[0], *ids[2:]]
        rows = [line for line in lines if not line.startswith('line\t')]
        rows[4:4] = [f"line\t{cue}\t0\t0\t{T.N / (cue + '.wav')}" for cue in wanted]
        path.write_text('\n'.join(rows) + '\n')
        item['cues'], item['timeline_sha256'] = wanted, sha(path)
    write_json(job / 'timeline-summary.json', summary)
    write_json(job / 'prior-table-comparison.json', compared)
    write_json(job / 'resource-receipt.json', receipt)
    (tmp / 'frozen.txt').write_text('frozen')
    write_json(tmp / 'timeline-inputs.json', {'files': [
        {'path': str(tmp / 'frozen.txt'), 'sha256': '0' * 64 if case == 'frozen-input' else sha(tmp / 'frozen.txt')},
        {'path': str(tmp / 'fake'), 'sha256': sha(tmp / 'fake')}]})
    write_json(tmp / 'config.json', {
        'case': case, 'chapters': list(T.CHAPTERS), 'narration': str(T.N), 'timeline_out': str(out), 'frames': FRAMES})
    (tmp / 'timeline-stage/controller01.exit').write_text('0\n')
    if case == 'duplicate-job':
        (tmp / 'B' / S.JOB).mkdir()


def fake_guard(tmp, argv):
    """What the real guard does around a child, with no resource sampling."""
    stage = next(name for name in STAGES if f'-{name}01' in argv[argv.index('--job') + 1])
    H = patched(stage, tmp)
    S, P = layers(stage, H)
    job = S.T.B / H.JOB
    job.mkdir()
    write_json(tmp / f'guard-{stage}.json', argv)
    case = json.loads((tmp / 'config.json').read_text())['case']
    if case == 'preview-file-changed' and stage == 'preview':
        with (S.OUT / 'stems/qa' / f'{S.T.CHAPTERS[0]}.json').open('a') as handle:
            handle.write('\n')
    if case == f'existing-{stage}':
        H.OUT.mkdir()
        (H.OUT / 'keep.txt').write_text('keep')
    command = argv[argv.index('--') + 1:]
    receipt = {'admitted': True, 'command': command,
               'policy': dict(S.T.POLICY, work_cap_gib=48) if case == f'guard-policy-{stage}' else S.T.POLICY}
    write_json(job / 'resource-receipt.json', receipt)
    with (job / 'command.log').open('x') as log:
        code = subprocess.run([sys.executable, '-I', '-B', str(HERE), '--child', stage, str(tmp)],
                              stdout=log, stderr=subprocess.STDOUT).returncode
    receipt.update(exit=code, finished='fixture', inputs_sha256='a', final_inputs_sha256='a')
    write_json(job / 'resource-receipt.json', receipt)
    return code


def stage_run(stage, tmp, jobs):
    """One controller as its start script runs it: wait, launch, then write its exit file."""
    with (tmp / f'{stage}.log').open('x') as log:
        code = subprocess.run([sys.executable, '-I', '-B', str(HERE), '--stage', stage, str(tmp)],
                              stdout=log, stderr=subprocess.STDOUT).returncode
    (tmp / 'downstream' / f'optimized-{stage}-controller01.exit').write_text(f'{code}\n')
    command_log = tmp / 'B' / jobs[stage] / 'command.log'
    return code, (tmp / f'{stage}.log').read_text() + (command_log.read_text() if command_log.exists() else '')


def between_stages(case, stage, tmp, jobs):
    """Damage made after a stage ended, before the next one starts."""
    B = tmp / 'B'
    if case == 'qa-edited' and stage == 'score':
        path = sorted((B / jobs['score'] / 'scored/stems/qa').glob('*.json'))[0]
        with path.open('a') as handle:
            handle.write('\n')
    if case == 'no-score-receipt' and stage == 'score':
        (B / jobs['score'] / 'resource-receipt.json').rename(tmp / 'moved.json')
    if case == 'lint-removed' and stage == 'preview':
        (B / jobs['preview'] / 'preview/15_closing.layout.tsv').rename(tmp / 'moved.tsv')
    if case == 'no-preview-receipt' and stage == 'preview':
        (B / jobs['preview'] / 'resource-receipt.json').rename(tmp / 'moved.json')


def refusal(case, wanted_stage, words, tmp, jobs):
    outcome = None
    for stage in STAGES:
        code, text = stage_run(stage, tmp, jobs)
        done = (tmp / 'B' / jobs[stage] / DONE[stage]).exists()
        if stage == wanted_stage:
            outcome = code != 0 and not done and words in text
            why = '' if outcome else f' code {code}, done {done}, ends {text.strip().splitlines()[-2:]}'
            next_stage = {'score': 'preview', 'preview': 'pictures'}.get(stage)
            if outcome and next_stage:
                # the next controller must see this exit file and launch nothing
                code, text = stage_run(next_stage, tmp, jobs)
                outcome = code != 0 and 'Upstream controller failed' in text and not (tmp / 'B' / jobs[next_stage]).exists()
                why = '' if outcome else f' next stage {code} {text[-200:]}'
            check(f'r-{case}', outcome, f'{stage} refused ({words}); no success file' + ('; the next controller launched nothing' if next_stage else '') + why)
            if case.startswith('existing-'):
                check(f'r-{case}-kept', [p.read_text() for p in (tmp / 'B' / jobs[stage]).rglob('keep.txt')] == ['keep'], 'the earlier output is still there')
            return
        if code != 0:
            check(f'r-{case}', False, f'{stage} failed early: {text.strip().splitlines()[-2:]}')
            return
        between_stages(case, stage, tmp, jobs)


def passing_chain(tmp, jobs):
    T = load('score').T
    for stage in STAGES:
        code, text = stage_run(stage, tmp, jobs)
        check(f'e-{stage}', code == 0 and (tmp / 'B' / jobs[stage] / DONE[stage]).exists(),
              f'wait, manifest, guard, child and success file; old exit files of 1 in the real tree were not consulted'
              + ('' if code == 0 else f' {text.strip().splitlines()[-3:]}'))
    B = tmp / 'B'
    argv = json.loads((tmp / 'guard-score.json').read_text())
    shown = [Path(x).name if x.startswith('/') and x != sys.executable else x for x in argv]
    check('e-guard-argv', shown == ['--phase', 'native', '--job', jobs['score'], '--inputs', 'optimized-score-inputs01.json',
                                    '--', sys.executable, '-I', '-B', 'score_optimized01.py', '--run'],
          'the guard gets the same phase and the new job, manifest and child')
    stage_folder = tmp / 'downstream'
    manifests = {s: {row['path'] for row in json.loads((stage_folder / f'optimized-{n}-inputs01.json').read_text())['files']}
                 for s, n in [('score', 'score'), ('preview', 'preview'), ('pictures', 'picture'), ('thumbnail', 'thumbnail')]}
    timeline_files = {str(p) for p in (B / T.JOB).rglob('*') if p.is_file()}
    score_files = {str(p) for p in (B / jobs['score']).rglob('*') if p.is_file() and p.name not in ('resource-receipt.json', 'command.log')}
    preview_files = {str(p) for p in (B / jobs['preview']).rglob('*') if p.is_file()}
    ok = (timeline_files <= manifests['score'] and str(tmp / 'frozen.txt') in manifests['score']
          and str(BUNDLE / 'start-optimized-score01.sh') in manifests['score'] and str(BUNDLE / 'score_optimized01.py') in manifests['score']
          and manifests['score'] <= manifests['preview'] and score_files <= manifests['preview']
          and manifests['preview'] <= manifests['pictures'] and preview_files <= manifests['pictures']
          and str(B / jobs['preview'] / 'preview-summary.json') in manifests['thumbnail'])
    check('e-manifests', ok, 'each manifest holds the frozen rows, the helper, its start script and every earlier job file')
    preview = json.loads((B / jobs['preview'] / 'preview-summary.json').read_text())
    check('e-preview-summary', len(preview['chapters']) == 15 and preview['tables_equal_to_timeline'] == ['timeline', 'cues']
          and preview['full_baseline_equivalence_established'] is False and preview['rendered_lint'] is True
          and sum(row['layout_findings'] for row in preview['chapters']) == 1 and preview['voice'] == 'ElevenLabs Frederick Surrey',
          '15 chapters, real ffprobe on each movie, the one planted lint row counted, no baseline claim')
    pictures = json.loads((B / jobs['pictures'] / 'pictures/extraction.json').read_text())
    sheets = {p.parent.name for p in (B / jobs['pictures'] / 'pictures').glob('*/sheet-*.png')}
    sources = {path for paths in pictures['layout_sources'].values() for path in paths}
    check('e-extraction', pictures['pictures_viewed'] is False and sheets == set(T.CHAPTERS)
          and sources == {str(B / jobs['preview'] / 'preview' / f'{c}.layout.tsv') for c in T.CHAPTERS}
          and any('layout.tsv' in reason for sample in pictures['samples'] for reason in sample['reasons']),
          f"{len(pictures['samples'])} real frames extracted with ffmpeg, a sheet per chapter, lint read from the rendered preview only, viewing not claimed")
    thumb = json.loads((B / jobs['thumbnail'] / 'output/thumbnail-receipt.json').read_text())
    check('e-thumbnail-receipt', thumb['visual_review'] == 'pending' and thumb['publication'] == 'not uploaded', 'decoded at 3840x2160; review and publication stay open')


def wait_cases(root, shared):
    def waited(name, exit_text, polls, late=None):
        tmp = root / name
        fixture('pass', tmp, shared)
        exit_file = tmp / 'timeline-stage/controller01.exit'
        exit_file.rename(tmp / 'moved.exit')
        if exit_text is not None:
            exit_file.write_text(exit_text)
        return tmp, subprocess.run([sys.executable, '-I', '-B', str(HERE), '--wait', str(tmp), str(polls), late or ''],
                                   capture_output=True, text=True)
    tmp, run = waited('wait-failed', '1\n', 2)
    check('w-upstream-failed', run.returncode != 0 and 'Upstream controller failed' in run.stderr and not list((tmp / 'downstream').iterdir()),
          'a timeline exit of 1 refuses before any manifest or job')
    tmp, run = waited('wait-expired', None, 2)
    check('w-expired', run.returncode != 0 and 'Wait expired' in run.stderr and not list((tmp / 'downstream').iterdir()),
          'no exit file: the wait ends with nothing launched, and a job in progress is not read')
    tmp, run = waited('wait-late', None, 3, 'late')
    check('w-late', run.returncode == 0 and 'polls 2' in run.stdout, 'the wait passes on the poll after the exit file appears')


def start_scripts(root):
    """The real scripts in a throwaway tree of the same depth, with a stand-in helper."""
    tree = root / 'start'
    folder = tree / 'a/b/c/d/e'
    folder.mkdir(parents=True)
    shutil.copy(BUNDLE / 'start-optimized-preview01.sh', folder)
    (folder / 'preview_optimized01.py').write_text("print('ran')\nraise SystemExit(3)\n")
    script = folder / 'start-optimized-preview01.sh'
    first = subprocess.run(['/bin/bash', str(script)])
    work = tree / 'narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/optimized-downstream01'
    log, code = work / 'optimized-preview-controller01.log', work / 'optimized-preview-controller01.exit'
    ok = first.returncode == 3 and log.read_text() == 'ran\n' and code.read_text() == '3\n'
    check('s-start', ok, "the script writes the helper's output and its exit code to the new names")
    code.write_text('marker')
    second = subprocess.run(['/bin/bash', str(script)], capture_output=True, text=True)
    check('s-second-start', second.returncode == 1 and 'refused' in second.stderr and log.read_text() == 'ran\n' and code.read_text() == 'marker',
          "a second start is refused and leaves the first start's log and exit file alone")


def real_cue_logs(root):
    """check_cue_log on a finished film's real tables: 13 chapters pass, three kinds of damage fail."""
    P = load('preview')
    S = P.S
    lane = S.T.B
    render, timeline = lane / 'logarithm-stage09-render-work02', lane / 'logarithm-stage09-timeline02/timeline-output'
    S.OUT = lane / 'logarithm-stage09-score02/scored'
    keys = [p.name[:-len('.cuelog.tsv')] for p in sorted(render.glob('*.cuelog.tsv'))]
    plain = segmented = 0
    for key in keys:
        frames = next(line.split('\t')[1] for line in P.normalized(render / f'{key}.timeline.tsv') if line.startswith('frames\t'))
        P.check_cue_log(key, int(frames), render / f'{key}.cuelog.tsv', timeline / f'{key}.cuelog.tsv')
        assert P.normalized(render / f'{key}.cuelog.tsv') != P.normalized(timeline / f'{key}.cuelog.tsv')
        segmented += len(P.score_files(key, int(frames))) > 1
        plain += len(P.score_files(key, int(frames))) == 1
    check('x-real-pass', len(keys) == 13, f'13 real logarithm chapters pass ({plain} whole stems, {segmented} segmented), and in all 13 plain equality would have failed')
    key = next(k for k in keys if (S.OUT / 'stems' / f'{k}.music.f32.parts.tsv').exists())
    frames = next(line.split('\t')[1] for line in P.normalized(render / f'{key}.timeline.tsv') if line.startswith('frames\t'))
    lines = (render / f'{key}.cuelog.tsv').read_text().splitlines(keepends=True)
    damage = {'score row dropped': lines[:-1], 'foreign row added': lines + [lines[-1]],
              'narration row changed': [line.replace('\t60\t', '\t30\t', 1) if line is lines[2] else line for line in lines]}
    names = lines[0].lstrip('# ').rstrip('\n').split('\t')
    fields = lines[-1].rstrip('\n').split('\t')
    moved = float(fields[5]) + 1 / 48000
    bits = struct.unpack('>Q', struct.pack('>d', moved))[0]
    for index in (1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12):
        changed = list(fields)
        changed[index] = '0' * 64 if index == 1 else '0x1' if index in (6, 8) else '-6.0' if index == 7 else repr(moved) if index == 5 else str(int(fields[index]) + 1)
        damage[names[index] + ' changed'] = lines[:-1] + ['\t'.join(changed) + '\n']
    changed = list(fields)
    changed[5], changed[6] = repr(moved), f'{bits:#018x}'
    damage['offset and its bits moved one sample together'] = lines[:-1] + ['\t'.join(changed) + '\n']
    for label, rows in damage.items():
        path = root / 'damaged.cuelog.tsv'
        path.write_text(''.join(rows))
        try:
            P.check_cue_log(key, int(frames), path, timeline / f'{key}.cuelog.tsv')
            check('x-real-damage', False, label + ' passed')
        except AssertionError as error:
            check('x-real-damage', 'differ' in str(error), label + ' on real tables is refused')


def real_check():
    """The controllers' read-only --check on the real tree."""
    for stage in ('thumbnail', 'pictures'):
        run = subprocess.run([sys.executable, '-I', '-B', str(BUNDLE / f'{stage}_optimized01.py'), '--check'], capture_output=True, text=True)
        report = json.loads(run.stdout) if run.returncode == 0 else {}
        check(f'c-real-check-{stage}', report.get('launched') is False and report.get('frozen_files', 0) > 5000,
              f"frozen inputs, guard policy and unused names hold, nothing launched{'' if run.returncode == 0 else run.stderr[-300:]}")


def preserved(T):
    native = [T.NATIVE / name for name in (
        'timeline-after-audio04.py', 'score-after-timeline01.py', 'preview-after-score01.py', 'pictures-after-preview01.py',
        'thumbnail-after-preview01.py', 'start-score01.sh', 'start-preview01.sh', 'start-pictures01.sh', 'start-thumbnail01.sh')]
    recovery = T.R / 'docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-optimized-recovery-review01/SHA256SUMS'
    files = [*native, recovery, *[p for p in T.FAILED.rglob('*') if p.is_file()]]
    return {str(p): sha(p) for p in files}


def main():
    S = load('score')
    T = S.T
    jobs = {stage: load(stage).JOB for stage in STAGES}
    before = preserved(T)
    old = {'complex-stage06-score01', 'complex-stage06-narrated-preview01', 'complex-stage06-preview-pictures01', 'complex-stage06-thumbnail01'}
    check('c-new-names', len(set(jobs.values())) == 4 and not old & set(jobs.values())
          and not any((T.B / job).exists() for job in jobs.values()) and not S.STAGE.exists(),
          'four new jobs, none of them an old name, and neither they nor the new stage folder exist')
    source = ''.join((BUNDLE / f'{stage}_optimized01.py').read_text() for stage in STAGES)
    stale = [word for word in ('.exit\'', 'inputs04', 'timeline-after-audio04.py\'', 'root-build01') if word in source.replace("controller01.exit'", '')]
    exits = {name: (T.NATIVE / name).read_text().strip() for name in ('timeline-controller04.exit', 'score-controller01.exit', 'preview-controller01.exit')}
    check('c-no-stale-exits', not stale and all(value == '1' for value in exits.values()) and T.T.JOB not in source and T.JOB == 'complex-stage06-optimized-timeline01',
          f'the old exit files really hold 1 and no controller names them, inputs04 or the debug job{stale}')
    check('c-timeouts', 'timeout=7200)' in (BUNDLE / 'score_optimized01.py').read_text() and 'timeout=14400)' in (BUNDLE / 'preview_optimized01.py').read_text()
          and 'RENDER_TIMEOUT_SECONDS' not in source and S.WAIT_POLLS * 20 == 172800,
          'score keeps 7200 s and preview keeps 14400 s; none inherits the recovery limit; only the wait is 48 h')
    real_check()
    with tempfile.TemporaryDirectory() as folder:
        root = Path(folder)
        shared = root / 'shared'
        shared.mkdir()
        movies(shared)
        tmp = root / 'pass'
        fixture('pass', tmp, shared)
        passing_chain(tmp, jobs)
        for case, (stage, words) in {**TIMELINE_CASES, **RUN_CASES}.items():
            tmp = root / case
            fixture(case, tmp, shared)
            refusal(case, stage, words, tmp, jobs)
        wait_cases(root, shared)
        start_scripts(root)
        real_cue_logs(root)
    check('p-preserved', preserved(T) == before, f'all {len(before)} old helper, sealed recovery and failed-run files are byte-identical')
    check('p-nothing-launched', not any((T.B / job).exists() for job in jobs.values()) and not S.STAGE.exists(),
          'the real job names and stage folder are still unused')
    print(f"RESULT {'PASS' if all(results) else 'FAIL'}, {results.count(False)} failed")
    return 0 if all(results) else 1


def wait_only(tmp, polls, late):
    S = patched('score', tmp)
    count = [0]

    def sleep(_):
        count[0] += 1
        if late:
            (tmp / 'timeline-stage/controller01.exit').write_text('0\n')
    S.WAIT_POLLS = polls
    S.time.sleep = sleep
    S.wait_for(S.TIMELINE_EXIT)
    print('polls', count[0] + 1)


if __name__ == '__main__':
    mode = sys.argv[1:2]
    if mode == ['--stage']:
        patched(sys.argv[2], Path(sys.argv[3])).wait_and_launch()
    elif mode == ['--child']:
        getattr(patched(sys.argv[2], Path(sys.argv[3])), RUN[sys.argv[2]])()
    elif mode == ['--guard']:
        sys.exit(fake_guard(Path(sys.argv[2]), sys.argv[3:]))
    elif mode == ['--wait']:
        wait_only(Path(sys.argv[2]), int(sys.argv[3]), sys.argv[4] if len(sys.argv) > 4 else '')
    else:
        sys.exit(main())
