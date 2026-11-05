#!/usr/bin/env python3
"""Rehearse the chain in a tiny temporary tree whose repository path is a symlink, as on ts1.

Real: the chain, its start script, the sealed qualifier run first through its own start script,
ts1's guard01.py logic and the preserved guard, the two env imports, ffmpeg, ffprobe, PIL and the
real cue ids. Fake: the binary, the narration verifier, the takes, the receipts of the macOS run
and sbh. In the fixture copies only, the hostname and the hash pins change. Nothing is written in
the repository. Run from the repository root:

    python3 -I -B <bundle>/checks/rehearse_chain.py
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import wave

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
PIPE = BUNDLE.parent
REAL = HERE.parents[7]
SPEC = importlib.util.spec_from_file_location('rehearse_qualify', PIPE / 'qualify01/checks/rehearse_qualify.py')
Q = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(Q)
CHAIN = 'docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-remote-ts1-pipeline01/chain01'
CLOSURE_REL = 'docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-remote-ts1-pipeline01/narration-closure01.json'
FAILED = Q.B + '/complex-stage06-timeline01'
FRAMES = 6
FAKE = r'''#!PYTHON -I
import hashlib, json, shutil, struct, sys
from pathlib import Path
here = Path(sys.argv[0]).resolve().parent
c = json.loads((here / 'fake-config.json').read_text())
case, args = c['case'], sys.argv[1:]
if args[0] == 'facts':
    sys.stdout.write(c['facts'])
elif args[0] == 'chapters':
    print('\n'.join(c['chapters']))
elif args[:2] == ['render', 'all'] and '--timeline-only' in args:
    assert args[2:7] == ['--timeline-only', '--fps', '60', '--narration', c['narration']]
    out = Path(args[args.index('--out') + 1])
    out.mkdir()
    for chapter in c['chapters']:
        ids = [cue for cue in c['cues'] if cue.startswith(chapter[:2] + '_')]
        wav = (c['physical_narration'] if case == 'physical-path' else c['narration']) + '/'
        rows = ['# explainer timeline v1', f'chapter\t{chapter}', 'fps\t60', f"frames\t{c['frames']}"]
        rows += [f"line\t{cue}\t{min(k, c['frames'] - 1)}\t0\t{wav}{cue}.wav" for k, cue in enumerate(ids)]
        rows.append('breath\t3\t0.05')
        (out / f'{chapter}.timeline.tsv').write_text('\n'.join(rows) + '\n')
        (out / f'{chapter}.cues.tsv').write_text(''.join(f'{cue}\t0.000\t1.000\n' for cue in ids))
        (out / f'{chapter}.cuelog.tsv').write_text('# path\tsha256\n' + ''.join(
            f"{wav}{cue}.wav\tx\t8\t{min(k, c['frames'] - 1)}\t60\t0.0\t0x0\t-\t-\t24000\t1\t1\t0\n" for k, cue in enumerate(ids)))
elif args[0] == 'score':
    out = Path(args[args.index('--out') + 1])
    (out / 'stems/qa').mkdir(parents=True)
    for n, chapter in enumerate(c['chapters']):
        stem = out / 'stems' / f'{chapter}.music.f32.wav'
        if n == 1:
            names = [f'{stem.name}.part000{k}.wav' for k in (0, 1)]
            for name in names:
                (stem.parent / name).write_text(name)
            stem.with_suffix('.parts.tsv').write_text('score-parts\t1\n' + ''.join(f'{k * 2400}\t2400\t48000\tx\t{name}\n' for k, name in enumerate(names)))
        else:
            stem.write_text(chapter)
        (stem.parent / 'qa' / f'{chapter}.json').write_text(json.dumps({
            'chapter': chapter, 'passed': not (case == 'qa-fail' and n == 3), 'metrics': [{'name': 'level', 'pass': True}, {'name': 'note', 'pass': None}]}))
elif args[:2] == ['render', 'all']:
    assert args[2:7] == ['--res', '960x540', '--fps', '60', '--narration'] and args[-5:] == ['--lint', '--crf', '18', '--preset', 'veryfast']
    stems, out = Path(args[args.index('--score') + 1]), Path(args[args.index('--out') + 1])
    out.mkdir()
    timeline = Path(c['timeline_out'])
    for n, chapter in enumerate(c['chapters']):
        for table in ('timeline', 'cues'):
            (out / f'{chapter}.{table}.tsv').write_text((timeline / f'{chapter}.{table}.tsv').read_text())
        stem = stems / f'{chapter}.music.f32.wav'
        parts = stem.with_suffix('.parts.tsv')
        files = [(stem, 0, c['frames'] * 800)]
        if parts.exists():
            files = [(stem.with_name(f[4]), int(f[0]), int(f[1])) for f in (line.split('\t') for line in parts.read_text().splitlines()[1:])]
        laid = ''
        for path, start, samples in files:
            offset = start / 48000 - c['frames'] / 60
            exact = struct.unpack('>Q', struct.pack('>d', offset))[0]
            shown = str(path.resolve()) if case == 'score-path-physical' and n == 5 else str(path)
            laid += '\t'.join([shown, hashlib.sha256(path.read_bytes()).hexdigest(), str(path.stat().st_size), str(c['frames']), '60', repr(offset),
                               f'{exact:#018x}', '0.0', '0x0000000000000000', '48000', '2', str(samples), str(start)]) + '\n'
        (out / f'{chapter}.cuelog.tsv').write_text((timeline / f'{chapter}.cuelog.tsv').read_text() + laid)
        if not (case == 'lint-missing' and n == 9):
            (out / f'{chapter}.layout.tsv').write_text('' if n else '01_1 +0.0s @0.04s\ttight\t"z" stands 0.20 from the edge\n')
        shutil.copy(here / 'fake-tiny.mp4', out / f'{chapter}.mp4')
'''
results = []
sha = Q.sha
write = Q.write


def check(name, ok, detail=''):
    results.append(bool(ok))
    detail = detail.replace(Q.FIXTURE[0], '<fixture>') if Q.FIXTURE[0] else detail
    print(f"{'PASS' if ok else 'FAIL'} {name}{': ' + detail if detail else ''}", flush=True)


class Chain(Q.Tree):
    """The qualifier's fixture, with the chain and the narration tree added."""

    def __init__(self, base, case, shared):
        super().__init__(base)
        self.case = case
        self.helper_chain = self.R / CHAIN / 'ts1_chain01.py'
        self.N = self.R / Q.EC / 'production-audio-stage06-01/native-narration01'
        self.workspace = self.R / Q.EC / 'native-stage06-root03/workspace'
        (self.R / CHAIN).mkdir(parents=True)
        shutil.copy(BUNDLE / 'start-chain01.sh', self.R / CHAIN / 'start-chain01.sh')
        shutil.copy(REAL / Q.CRATE / 'script.tsv', self.C / 'script.tsv')
        self.cues = [line.split('\t', 1)[0] for line in (self.C / 'script.tsv').read_text().splitlines() if line.strip() and not line.startswith('#')]
        self.narration()
        write(self.workspace / 'tools/coherent_narration.py', "print('fake verifier')\n")
        env = self.J / 'complex-remote-picture-env01'
        (env / 'env/bin').mkdir(parents=True)
        (env / 'env/bin/python').symlink_to(sys.executable)
        write(env / 'resource-receipt.json', json.dumps(Q.receipt('env', [], 'x', 0)))
        write(self.O / 'fake-config.json', json.dumps({
            'case': case, 'facts': Q.FACTS, 'chapters': Q.CHAPTERS, 'cues': self.cues, 'frames': FRAMES, 'narration': str(self.N),
            'physical_narration': str(self.N.resolve()), 'timeline_out': str(self.J / 'complex-remote-timeline01/timeline-output')}))
        shutil.copy(shared / 'tiny.mp4', self.O / 'fake-tiny.mp4')
        text = FAKE.replace('PYTHON', sys.executable)
        for path in (self.O / 'complex_analysis_visual-release01', self.O / 'target/release/complex_analysis_visual'):
            write(path, text, 0o755)
        self.failed()
        self.lists()
        self.pin()
        self.pin_chain()

    def narration(self):
        phrases, lines = {}, {}
        for cue in self.cues:
            path = self.N / f'{cue}.wav'
            path.parent.mkdir(parents=True, exist_ok=True)
            with wave.open(str(path), 'wb') as handle:
                handle.setnchannels(1)
                handle.setsampwidth(2)
                handle.setframerate(24000)
                handle.writeframes(b'\0\0' * 240)
            phrases[cue] = {'wav_sha256': sha(path), 'marks': [{'seconds': 0.01, 'byte': 0}]}
            lines[cue] = [{'start': 0.0}]
        write(self.N / 'phrase-timings.json', json.dumps({'cues': phrases}))
        write(self.N / 'report.json', json.dumps({'lines': lines}))
        write(self.N / 'external-units.json', '{}')

    def failed(self):
        job = self.R / FAILED
        out = job / 'timeline-output'
        out.parent.mkdir(parents=True)
        subprocess.run([str(self.O / 'complex_analysis_visual-release01'), 'render', 'all', '--timeline-only', '--fps', '60', '--narration', str(self.N), '--out', str(out)], check=True)
        for chapter in Q.CHAPTERS[4:]:
            for table in ('timeline', 'cues', 'cuelog'):
                (out / f'{chapter}.{table}.tsv').unlink()
        write(job / 'facts-native.stdout', Q.FACTS)
        log = 'timed out after 14400 seconds\n' + ''.join(f'{chapter}  {FRAMES} frames  1.0s\n' for chapter in Q.CHAPTERS[:4])
        write(job / 'command.log', log)
        write(job / 'resource-receipt.json', json.dumps(dict(Q.receipt('failed', [], '7d6aeaf7d1f8b83a55fb7818f48ff01c2c6fa62ba135d84ea63c8a087307072c', 1))))

    def lists(self):
        paths = [p for top in (self.N, self.workspace, self.R / FAILED) for p in top.rglob('*') if p.is_file()]
        known = {row['path'] for row in json.loads((self.O / 'inputs01.json').read_text())['files']}
        closure = self.R / 'closure-row.bin'
        write(closure, 'a take the verifier opens\n')
        write(self.O / 'audio-support-transfer01.json', json.dumps({'files': [{'path': str(p), 'sha256': sha(p)} for p in paths if str(p) not in known]}))
        write(self.R / CLOSURE_REL, json.dumps({'files': [{'path': 'closure-row.bin', 'sha256': sha(closure), 'bytes': 1}]}))

    def pin_chain(self):
        text = (BUNDLE / 'ts1_chain01.py').read_text()
        for name, path in (('QUALIFIER_SHA256', self.helper), ('AUDIO_ROWS_SHA256', self.O / 'audio-support-transfer01.json'), ('CLOSURE_SHA256', self.R / CLOSURE_REL)):
            old = text.split(f"{name} = '")[1].split("'")[0]
            text = text.replace(old, sha(path))
        import PIL
        text = text.replace("PILLOW = '12.1.1'", f"PILLOW = '{PIL.__version__}'")
        self.helper_chain.write_text(text)

    def pin(self):
        super().pin()

    def qualify(self):
        digest = sha(self.O / 'inputs01.json')
        self.receipt.update(inputs_sha256=digest, final_inputs_sha256=digest)
        self.save()
        run = self.start()
        assert run.returncode == 0, (self.O / 'qualify-controller01.log').read_text()[-400:]

    def stage(self, name):
        return subprocess.run(['bash', str(self.R / CHAIN / 'start-chain01.sh'), name], env=self.env, capture_output=True, text=True)

    def chain_check(self, name):
        return subprocess.run([self.python, '-I', '-B', str(self.helper_chain), '--check', name], cwd=self.R, env=self.env, capture_output=True, text=True)

    def text(self, stage):
        logs = [self.O / f'chain-{stage}-controller01.log', self.J / f'complex-remote-{stage}01/command.log']
        return '\n'.join(path.read_text() for path in logs if path.exists())


def last(text, empty='no output'):
    return Q.last_line(text, empty)


def summary_edit(key, value):
    def apply(t):
        path = t.J / 'complex-remote-qualify01/qualification-summary.json'
        d = json.loads(path.read_text())
        d[key] = value
        path.write_text(json.dumps(d))
    return apply


def append(relative):
    def apply(t):
        with (t.R / relative).open('a') as handle:
            handle.write('edit\n')
    return apply


def extra_take(t):
    write(t.N / 'stray.wav', 'stray')


def closure_gone(t):
    (t.R / 'closure-row.bin').rename(t.R / 'closure-row.moved')


def mac_tables_differ(t):
    path = t.R / FAILED / 'timeline-output/02_turning_numbers.cues.tsv'
    path.write_text(path.read_text() + 'extra\t0.000\t1.000\n')
    t.lists()
    t.pin_chain()


def mac_facts_differ(t):
    write(t.R / FAILED / 'facts-native.stdout', 'c\t3\n')
    t.lists()
    t.pin_chain()


def qualifier_failed(t):
    write(t.O / 'qualify-controller01.exit', '1\n')


# name: (when the damage lands, stage that must refuse, words)
BEFORE_QUALIFY = {}
AFTER_QUALIFY = {
    'qualifier-exit': (qualifier_failed, 'timeline', 'Upstream controller failed'),
    'relabelled-build': (summary_edit('original_build_relabelled', True), 'timeline', 'Original build was relabelled'),
    'relabelled-exit': (summary_edit('original_build_exit', 0), 'timeline', 'Original build was relabelled'),
    'qualified-other-binary': (summary_edit('binary_sha256', '0' * 64), 'timeline', 'Qualified binary differs'),
    'original-receipt-edited': (summary_edit('original_receipt_sha256', '0' * 64), 'timeline', 'Original evidence changed'),
    'original-log-edited': (summary_edit('original_log_sha256', '0' * 64), 'timeline', 'Original evidence changed'),
    'audio-rows-changed': (append(Q.EC + '/remote-ts1-01/audio-support-transfer01.json'), 'timeline', 'Audio transfer rows changed'),
    'closure-list-changed': (append(CLOSURE_REL), 'timeline', 'Narration closure changed'),
    'closure-file-missing': (closure_gone, 'timeline', 'closure-row.bin'),
    'stray-take': (extra_take, 'timeline', 'File set changed'),
    'macos-tables-differ': (mac_tables_differ, 'timeline', 'ts1 tables differ from the macOS run'),
    'macos-facts-differ': (mac_facts_differ, 'timeline', 'ts1 facts differ from macOS'),
}
FAKE_CASES = {
    'physical-path': ('timeline', 'ts1 tables differ from the macOS run'),
    'qa-fail': ('score', 'chapter'),
    'score-path-physical': ('preview', 'Preview score rows differ: 06_'),
    'lint-missing': ('preview', 'Rendered lint report missing: 10_'),
}


def fresh(root, name, shared, case='baseline'):
    base = root / name
    base.mkdir()
    return Chain(base.resolve(), case, shared)


def main():
    with tempfile.TemporaryDirectory(prefix='chain-') as folder:
        root = Path(folder).resolve()
        Q.FIXTURE[0] = str(root)
        shared = root / 'shared'
        shared.mkdir()
        subprocess.run(['ffmpeg', '-v', 'error', '-nostdin', '-f', 'lavfi', '-i', 'color=c=black:s=960x540:r=60', '-f', 'lavfi', '-i', 'anullsrc=r=48000:cl=stereo',
                        '-frames:v', str(FRAMES), '-t', '0.1', '-c:v', 'libx264', '-preset', 'ultrafast', '-pix_fmt', 'yuv420p', str(shared / 'tiny.mp4')], check=True)
        t = fresh(root, 'baseline', shared)
        run = t.chain_check('pictures')
        check('check-before-qualify', run.returncode == 0 and '"launched": false' in run.stdout, last(run.stderr, 'ok'))
        t.qualify()
        check('qualifier-first', json.loads((t.J / 'complex-remote-qualify01/qualification-summary.json').read_text())['original_build_exit'] == 1, 'the sealed qualifier ran through its own start script')
        for stage in ('timeline', 'score', 'preview', 'pictures'):
            run = t.stage(stage)
            d = json.loads((t.J / f'complex-remote-{stage}01/resource-receipt.json').read_text())
            check('stage-' + stage, run.returncode == 0 and d['exit'] == 0 and d['inputs_sha256'] == d['final_inputs_sha256'] and d['policy'] == Q.POLICY, last(t.text(stage)) if run.returncode else 'the real guard admitted and closed the job')
        s = json.loads((t.J / 'complex-remote-timeline01/timeline-summary.json').read_text())
        check('timeline-summary', s['full_baseline_equivalence_established'] is False and s['prior_tables_compared'] == 12 and s['cues'] == 105 and s['qualifier_job'] == 'complex-remote-qualify01', 'no full baseline equivalence is claimed')
        compared = json.loads((t.J / 'complex-remote-timeline01/prior-table-comparison.json').read_text())
        check('prior-tables-line-exact', all(row['normalized_equal'] for row in compared['tables']) and len(compared['tables']) == 12, 'logical paths keep tables equal to the macOS strings')
        x = json.loads((t.J / 'complex-remote-pictures01/pictures/extraction.json').read_text())
        sheets = list((t.J / 'complex-remote-pictures01/pictures').rglob('sheet-*.png'))
        check('pictures-not-viewed', x['pictures_viewed'] is False and x['samples'] and len({sample['chapter'] for sample in x['samples']}) == 15 and sheets, f"{len(x['samples'])} pictures, {len(sheets)} sheets, none marked viewed")
        rows = {row['path'] for row in json.loads((t.O / 'chain-pictures-inputs01.json').read_text())['files']}
        wanted = [t.J / 'complex-remote-build01/resource-receipt.json', t.J / 'complex-remote-qualify01/qualification-summary.json', t.N / 'report.json', t.R / 'closure-row.bin',
                  t.J / 'complex-remote-preview01/preview-summary.json', t.helper_chain]
        check('last-manifest-carries-chain', all(str(p) in rows for p in wanted) and not [p for p in rows if p.startswith(str(t.physical))], 'evidence from every stage reaches the last manifest, by logical paths')
        run = t.stage('timeline')
        check('second-start-refused', run.returncode == 1 and 'refused' in run.stderr)
        run = subprocess.run(['bash', str(t.R / CHAIN / 'start-chain01.sh'), 'nonsense'], env=t.env, capture_output=True, text=True)
        check('unknown-stage-refused', run.returncode == 2 and 'usage' in run.stderr)
        for name, (damage, stage, words) in AFTER_QUALIFY.items():
            t = fresh(root, name, shared)
            t.qualify()
            damage(t)
            run = t.stage(stage)
            text = t.text(stage)
            check('refuse-' + name, run.returncode != 0 and words in text, last(text))
        for case, (stage, words) in FAKE_CASES.items():
            t = fresh(root, case, shared, case)
            t.qualify()
            order = ('timeline', 'score', 'preview')
            for name in order[:order.index(stage)]:
                assert t.stage(name).returncode == 0, name
            run = t.stage(stage)
            text = t.text(stage)
            check('stop-' + case, run.returncode != 0 and words in text, last(text))
    print(f"RESULT {'PASS' if all(results) else 'FAIL'}, {results.count(False)} failed")
    return 0 if all(results) else 1


if __name__ == '__main__':
    sys.exit(main())
