#!/usr/bin/env python3
"""Rehearse the N2 recovery on the real narration, in a tree whose repository path is a symlink.

Real: all 105 cues, their takes and receipts, the unmodified coherent_narration.py and
narration_delivery.py, the sealed qualifier and chain01, the real guard, ffmpeg, ffprobe and PIL.
Fake: the binary, which makes the kit's exact narration call and then writes tiny tables, plus the
macOS run's receipts and sbh. report.json is respelled into the fixture root, as ts1's is in /Users.

Takes are APFS clones, so they use no space and share no inode with production. Run from the
repository root on the local host:

    python3 -I -B <bundle>/checks/rehearse_n2.py
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
SPEC = importlib.util.spec_from_file_location('rehearse_chain', BUNDLE.parent / 'chain01/checks/rehearse_chain.py')
C = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(C)
Q = C.Q
REAL = C.REAL
FOLDER = C.CHAIN.replace('/chain01', '/chain02-n2')
N_REL = Q.EC + '/production-audio-stage06-01/native-narration01'
WORKSPACE_REL = Q.EC + '/native-stage06-root03/workspace'
DERIVE = 'complex-remote-narration-derive01'
REFUSAL = 'coherent-narration: assembly sentence/source spans differ from the supplied script and generation receipts'
sha = Q.sha

DELIVER = r'''
import subprocess
def deliver(consumer, narration, more=()):
    if not c.get('deliver'):
        return
    keys = [part for chapter in c['chapters'] for part in ('--chapter-key', chapter)]
    call = ['python3', '-I', '-B', c['tool'], '--topic', c['topic'], '--narration', narration, '--consumer', consumer, '--gain-db', '0', *keys, *more]
    if subprocess.call(call) != 0:
        print('complex_analysis_visual: narration delivery contract refused; no consumer output was written', file=sys.stderr)
        raise SystemExit(1)
'''
EDITS = [
    ("case, args = c['case'], sys.argv[1:]\n", "case, args = c['case'], sys.argv[1:]\n" + DELIVER),
    ("    assert args[2:7] == ['--timeline-only', '--fps', '60', '--narration', c['narration']]\n",
     "    narration = args[args.index('--narration') + 1]\n    deliver('render', narration)\n"),
    ("wav = (c['physical_narration'] if case == 'physical-path' else c['narration']) + '/'",
     "wav = (str(Path(narration).resolve()) if case == 'physical-path' else c['narration'] if case == 'n1-spelling' else narration) + '/'"),
    (r'''f"{wav}{cue}.wav\tx\t8\t''', r'''f"{wav}{cue}.wav\t{'y' if case == 'hash-cell' and k == 2 else 'x'}\t8\t'''),
    ("elif args[0] == 'score':\n", "elif args[0] == 'score':\n    deliver('score', args[args.index('--narration') + 1], ['--timelines', args[args.index('--timelines') + 1]])\n"),
    ("    stems, out = Path(args[args.index('--score') + 1]), Path(args[args.index('--out') + 1])\n",
     "    deliver('render', args[args.index('--narration') + 1], ['--score', args[args.index('--score') + 1]])\n    stems, out = Path(args[args.index('--score') + 1]), Path(args[args.index('--out') + 1])\n"),
]
for old, new in EDITS:
    assert C.FAKE.count(old) == 1, old
    C.FAKE = C.FAKE.replace(old, new)


def closure_paths():
    return [row['path'] for row in json.loads((REAL / C.CLOSURE_REL).read_text())['files']]


class Real(C.Chain):
    deliver = False

    def narration(self):
        rels = [str(p.relative_to(REAL)) for top in (N_REL, WORKSPACE_REL) for p in (REAL / top).rglob('*') if p.is_file()]
        dispositions = json.loads((REAL / N_REL / 'report.json').read_text())['meta']['direction_contract']['dispositions']
        self.closure = [*closure_paths(), *sorted({row['evidence']['path'] for row in dispositions})]
        for rel in [*rels, *self.closure]:
            dst = self.R / rel
            if dst.exists() or dst.name in ('report.json', 'assembly.json') and rel.startswith(N_REL):
                continue
            dst.parent.mkdir(parents=True, exist_ok=True)
            if dst.suffix == '.wav':
                subprocess.run(['/bin/cp', '-c', str(REAL / rel), str(dst)], check=True)
            else:
                shutil.copyfile(REAL / rel, dst)
        report = (REAL / N_REL / 'report.json').read_bytes()
        old = hashlib.sha256(report).hexdigest()
        report = report.replace(str(REAL).encode(), str(self.R).encode())
        (self.N / 'report.json').write_bytes(report)
        assembly = (REAL / N_REL / 'assembly.json').read_bytes().replace(old.encode(), hashlib.sha256(report).hexdigest().encode())
        (self.N / 'assembly.json').write_bytes(assembly)

    def config(self, case):
        super().config(case)
        path = self.O / 'fake-config.json'
        data = json.loads(path.read_text())
        data.update(deliver=self.deliver, tool=str(self.workspace / 'tools/narration_delivery.py'), topic=str(self.C),
                    timeline_out=str(self.J / 'complex-remote-timeline02/timeline-output'))
        path.write_text(json.dumps(data))

    def lists(self):
        super().lists()
        path = self.R / C.CLOSURE_REL
        data = json.loads(path.read_text())
        data['files'] += [{'path': rel, 'sha256': sha(self.R / rel), 'bytes': 0} for rel in self.closure]
        path.write_text(json.dumps(data))

    def pin_n2(self):
        folder = self.R / FOLDER
        folder.mkdir(exist_ok=True)
        shutil.copy(BUNDLE / 'start-chain02-n2.sh', folder)
        text = (BUNDLE / 'ts1_chain02_n2.py').read_text()
        old = text.split("CHAIN01_SHA256 = '")[1].split("'")[0]
        self.script = folder / 'ts1_chain02_n2.py'
        self.script.write_text(text.replace(old, sha(self.helper_chain)))

    def repin(self):
        self.lists()
        self.pin_chain()
        self.pin_n2()

    def finish(self):
        shutil.copyfile(REAL / WORKSPACE_REL / 'tools/coherent_narration.py', self.workspace / 'tools/coherent_narration.py')
        self.deliver = True
        self.config(self.case)
        self.lists()
        self.pin()
        self.pin_chain()
        self.pin_n2()

    def start2(self, stage):
        return subprocess.run(['bash', str(self.R / FOLDER / 'start-chain02-n2.sh'), stage], env=self.env, capture_output=True, text=True)

    def mode2(self, mode, name):
        return subprocess.run([self.python, '-I', '-B', str(self.script), mode, name], cwd=self.R, env=self.env, capture_output=True, text=True)

    def text2(self, stage):
        job = DERIVE if stage == 'derive' else f'complex-remote-{stage}02'
        logs = [self.O / f'chain-{stage}-controller02.log', self.J / job / 'command.log']
        return '\n'.join(path.read_text() for path in logs if path.exists())

    @property
    def n2(self):
        return self.J / DERIVE / 'narration-n2'


def fresh(root, name, shared, case='baseline'):
    base = root / name
    base.mkdir()
    t = Real(base.resolve(), case, shared)
    t.finish()
    t.qualify()
    return t, t.stage('timeline')


def edited(t, change):
    """Run a damaged N as if it had been transferred that way, and put it back."""
    saved = {name: (t.N / name).read_bytes() for name in ('report.json', 'assembly.json')}
    report = json.loads(saved['report.json'])
    change(report)
    text = json.dumps(report).encode()
    (t.N / 'report.json').write_bytes(text)
    (t.N / 'assembly.json').write_bytes(saved['assembly.json'].replace(hashlib.sha256(saved['report.json']).hexdigest().encode(), hashlib.sha256(text).hexdigest().encode()))
    t.repin()
    run = t.mode2('--check', 'derive')
    for name, data in saved.items():
        (t.N / name).write_bytes(data)
    t.repin()
    return run


def other_take(report):
    report['lines']['01_1'][0]['source'] = report['lines']['01_2'][0]['source']


def boundary(report):
    report['lines']['01_1'][0]['source_out_sample'] += 1


def row_missing(report):
    report['lines']['03_4'].pop()


def extra_key(report):
    report['lines']['01_1'][0]['extra'] = 1


def flip(path):
    data = bytearray(path.read_bytes())
    data[len(data) // 2] ^= 1
    path.write_bytes(data)
    return bytes(data)


def main():
    with tempfile.TemporaryDirectory(prefix='n2-') as folder:
        root = Path(folder).resolve()
        Q.FIXTURE[0] = str(root)
        shared = root / 'shared'
        shared.mkdir()
        subprocess.run(['ffmpeg', '-v', 'error', '-nostdin', '-f', 'lavfi', '-i', 'color=c=black:s=960x540:r=60', '-f', 'lavfi', '-i', 'anullsrc=r=48000:cl=stereo',
                        '-frames:v', str(C.FRAMES), '-t', '0.1', '-c:v', 'libx264', '-preset', 'ultrafast', '-pix_fmt', 'yuv420p', str(shared / 'tiny.mp4')], check=True)
        t, run = fresh(root, 'baseline', shared)
        old = (t.J / 'complex-remote-timeline01/command.log').read_text()
        C.check('sealed-chain01-reproduces-the-refusal', run.returncode != 0 and old.count(REFUSAL) == 1 and not (t.J / 'complex-remote-timeline01/timeline-output').exists(),
                'real tool, real 105 cues, real guard, symlinked root')
        before = {p.name: sha(p) for p in t.N.iterdir()}
        run = t.mode2('--check', 'derive')
        lines = run.stdout.strip().splitlines()
        dry = json.loads(lines[-1]) if run.returncode == 0 else {}
        C.check('check-derive-dry-run', run.returncode == 0 and dry['derivation'] == {'cues': 105, 'rows': 414, 'source_fields_changed': 414, 'other_fields_changed': 0, 'samefile': 414, 'take_hash_equal': 414, 'takes': 105},
                C.last(run.stderr, '105 cues, 414 rows, 414 source fields, 0 others'))
        C.check('dry-run-both-tools', run.returncode == 0 and dry['tools']['standalone_refuses_N'] and dry['tools']['delivery_cues'] == 105 and dry['tools']['plan_mode'] == 'external-ui', 'N refused; N2 passed by both unmodified tools')
        C.check('check-spends-no-name', not (t.J / DERIVE).exists() and not (t.O / 'chain-derive-inputs02.json').exists())
        for name, change, words in (('other-take', other_take, 'Not the same file: 01_1'), ('boundary-moved', boundary, 'More than source differs: 01_1'),
                                    ('row-missing', row_missing, 'Row count differs: 03_4'), ('row-key-added', extra_key, 'Row keys differ: 01_1')):
            run = edited(t, change)
            C.check('refuse-' + name, run.returncode != 0 and words in run.stderr, C.last(run.stderr, 'no output'))
        take = t.R / Q.EC / 'production-audio-stage06-01/decoded01/decoded/01_1.wav'
        saved = take.read_bytes()
        flip(take)
        t.repin()
        run = t.mode2('--check', 'derive')
        C.check('refuse-take-bytes-changed', run.returncode != 0 and 'external source, text or decoded audio changed' in run.stderr, C.last(run.stderr, 'no output'))
        take.write_bytes(saved)
        t.repin()
        wrong = (t.N / 'assembly.json').read_bytes()
        (t.N / 'assembly.json').write_bytes(wrong.replace(b'"report_sha256":"', b'"report_sha256":"0'))
        t.repin()
        run = t.mode2('--check', 'derive')
        C.check('refuse-stale-assembly', run.returncode != 0 and 'Assembly does not bind this report' in run.stderr, C.last(run.stderr, 'no output'))
        (t.N / 'assembly.json').write_bytes(wrong)
        t.repin()
        for stage in ('derive', 'timeline', 'score', 'preview', 'pictures'):
            run = t.start2(stage)
            job = DERIVE if stage == 'derive' else f'complex-remote-{stage}02'
            d = json.loads((t.J / job / 'resource-receipt.json').read_text())
            C.check('stage-' + stage, run.returncode == 0 and d['exit'] == 0 and d['inputs_sha256'] == d['final_inputs_sha256'] and d['policy'] == Q.POLICY,
                    C.last(t.text2(stage), 'no output') if run.returncode else 'the real guard admitted and closed the job')
        C.check('N-untouched', before == {p.name: sha(p) for p in t.N.iterdir()}, '110 files')
        same = [p.name for p in t.N.iterdir() if sha(p) == sha(t.n2 / p.name)]
        C.check('108-of-110-identical', len(same) == 108 and len(list(t.n2.iterdir())) == 110 and {p.name for p in t.N.iterdir()} - set(same) == {'report.json', 'assembly.json'})
        a, b = (t.N / 'report.json').read_bytes(), (t.n2 / 'report.json').read_bytes()
        C.check('report-only-source-bytes', b.count(json.dumps(str(t.physical)).encode()[:-1]) == 414 and b.replace(str(t.physical).encode(), str(t.R).encode()) == a, '414 real-root spellings; respelling them restores N byte for byte')
        x, y = (t.N / 'assembly.json').read_bytes(), (t.n2 / 'assembly.json').read_bytes()
        C.check('assembly-only-report-hash', y.replace(hashlib.sha256(b).hexdigest().encode(), hashlib.sha256(a).hexdigest().encode()) == x and x != y)
        s = json.loads((t.J / DERIVE / 'derive-summary.json').read_text())
        C.check('derive-summary', (s['cues'], s['rows'], s['source_fields_changed'], s['other_fields_changed'], s['files_identical'], len(s['source_map'])) == (105, 414, 414, 0, 108, 105))
        compared = json.loads((t.J / 'complex-remote-timeline02/prior-table-comparison.json').read_text())
        C.check('prior-tables-exact-mapping', all(row['normalized_equal'] for row in compared['tables']) and sum(row['path_cells_mapped'] for row in compared['tables']) == 52 and len(compared['tables']) == 12, '12 tables, 52 path cells mapped, every other cell equal')
        table = (t.J / 'complex-remote-timeline02/timeline-output/01_the_bump.timeline.tsv').read_text()
        C.check('tables-spell-N2', f"{t.n2}/01_1.wav" in table and str(t.N) not in table)
        run = t.mode2('--compare', '01_the_bump')
        rows = [json.loads(line) for line in run.stdout.splitlines()]
        C.check('compare-one-chapter', run.returncode == 0 and [r['normalized_equal'] for r in rows] == [True] * 3, f"{sum(r['path_cells_mapped'] for r in rows)} path cells in chapter 01")
        rows = {row['path'] for row in json.loads((t.O / 'chain-pictures-inputs02.json').read_text())['files']}
        wanted = [t.N / 'report.json', t.n2 / 'report.json', t.J / 'complex-remote-timeline01/command.log', t.R / C.CHAIN / 'ts1_chain01.py', t.script, t.J / 'complex-remote-qualify01/qualification-summary.json', t.J / 'complex-remote-preview02/preview-summary.json']
        C.check('last-manifest-carries-both-folders', all(str(p) in rows for p in wanted) and not [p for p in rows if p.startswith(str(t.physical))], 'N, N2, the failed job, chain01 and every stage, all by logical paths')
        first = {row['path']: row['sha256'] for row in json.loads((t.O / 'chain-timeline-inputs02.json').read_text())['files']}
        derived = [p for p in t.n2.iterdir()]
        C.check('timeline-manifest-freezes-N2', len(derived) == 110 and all(first.get(str(p)) == sha(p) for p in derived) and first.get(str(t.J / DERIVE / 'derive-summary.json')) == sha(t.J / DERIVE / 'derive-summary.json')
                and all(first.get(str(p)) == sha(p) for p in t.N.iterdir()), 'all 110 N2 rows, the summary and all 110 N rows, by hash')
        race = (f"import importlib.util, json, sys\nspec = importlib.util.spec_from_file_location('c', {str(t.script)!r})\nm = importlib.util.module_from_spec(spec)\nspec.loader.exec_module(m)\n"
                "calls = []\ndef flaky(stage):\n    calls.append(stage)\n    if len(calls) < 3:\n        raise json.JSONDecodeError('half written', '', 0)\n    return 'gate ran'\n"
                "m.sealed_child_gate = flaky\nprint(m.child_gate('derive'), len(calls))\n"
                "def wrong(stage):\n    raise AssertionError('real gate failure')\nm.sealed_child_gate = wrong\nm.child_gate('derive')\n")
        run = subprocess.run([t.python, '-I', '-B', '-c', race], cwd=t.R, env=t.env, capture_output=True, text=True)
        C.check('receipt-reread-bounded', run.stdout.strip() == 'gate ran 3' and 'real gate failure' in run.stderr, 'a half-written read is retried; a failed gate is not')
        run = t.start2('timeline')
        C.check('second-start-refused', run.returncode == 1 and 'refused' in run.stderr)
        run = subprocess.run(['bash', str(t.R / FOLDER / 'start-chain02-n2.sh'), 'nonsense'], env=t.env, capture_output=True, text=True)
        C.check('unknown-stage-refused', run.returncode == 2 and 'usage' in run.stderr)
        t, _ = fresh(root, 'tamper', shared)
        assert t.start2('derive').returncode == 0, t.text2('derive')
        wav, report = t.n2 / '01_1.wav', t.n2 / 'report.json'
        saved = wav.read_bytes()
        flip(wav)
        run = t.mode2('--launch', 'timeline')
        C.check('refuse-n2-wav-edited', run.returncode != 0 and 'N2 differs: 01_1.wav' in run.stderr, C.last(run.stderr, 'no output'))
        wav.write_bytes(saved)
        saved = report.read_bytes()
        report.write_bytes(saved.replace(b'"end":2.92', b'"end":2.93'))
        run = t.mode2('--launch', 'timeline')
        C.check('refuse-n2-timing-edited', run.returncode != 0 and 'N2 differs: report.json' in run.stderr, C.last(run.stderr, 'no output'))
        report.write_bytes(saved)
        (t.n2 / 'stray.wav').write_bytes(b'x')
        run = t.mode2('--launch', 'timeline')
        C.check('refuse-n2-extra-file', run.returncode != 0 and 'N2 file set differs' in run.stderr, C.last(run.stderr, 'no output'))
        (t.n2 / 'stray.wav').rename(t.base / 'stray.moved')
        receipt = t.J / 'complex-remote-timeline01/resource-receipt.json'
        saved = receipt.read_text()
        receipt.write_text(saved.replace('"exit": 1', '"exit": 0'))
        run = t.mode2('--check', 'timeline')
        C.check('refuse-timeline01-relabelled', run.returncode != 0 and 'timeline01 receipt changed' in run.stderr, C.last(run.stderr, 'no output'))
        receipt.write_text(saved)
        log = t.J / 'complex-remote-timeline01/command.log'
        saved = log.read_text()
        log.write_text(saved.replace(REFUSAL, 'something else'))
        run = t.mode2('--check', 'timeline')
        C.check('refuse-timeline01-other-failure', run.returncode != 0 and 'timeline01 log lacks the exact refusal' in run.stderr, C.last(run.stderr, 'no output'))
        log.write_text(saved)
        with t.helper_chain.open('a') as handle:
            handle.write('# edit\n')
        run = t.mode2('--check', 'timeline')
        C.check('refuse-chain01-edited', run.returncode != 0 and 'Sealed chain01 changed' in run.stderr, C.last(run.stderr, 'no output'))
        for case, words in (('physical-path', 'ts1 tables differ from the macOS run'), ('hash-cell', 'ts1 tables differ from the macOS run'), ('n1-spelling', 'Mapped path cells differ')):
            t, _ = fresh(root, case, shared, case)
            assert t.start2('derive').returncode == 0, t.text2('derive')
            run = t.start2('timeline')
            text = t.text2('timeline')
            evidence = (t.J / 'complex-remote-timeline02/prior-table-comparison.json').exists()
            C.check('stop-table-' + case, run.returncode != 0 and words in text and evidence, C.last(text, 'no output'))
    print(f"RESULT {'PASS' if all(C.results) else 'FAIL'}, {C.results.count(False)} failed")
    return 0 if all(C.results) else 1


if __name__ == '__main__':
    sys.exit(main())
