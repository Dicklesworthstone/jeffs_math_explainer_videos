#!/usr/bin/env python3
"""Rehearse chain03 in a tree whose repository path is a symlink, as on ts1, after the jobs that really come first.

The sealed qualifier, chain01's refusal and chain02 run first, so timeline02 is a baseline those tools made. Then the
stage07 audio goes in and qualify02 and chain03 run through their start scripts.

Real: all 105 cues of the stage07 narration as rehearse_stage07.py assembled them on this Mac, with the real take
root chose provisionally; root's receipts; the stage07 tree; both unmodified narration tools; the sealed qualifier, chain01
and chain02; the real guard; ffmpeg, ffprobe and PIL. Fake: the binary, which makes the kit's narration call and writes
tiny tables in the kit's row order and arithmetic, and rustup, cargo, the lock tool, the oracle and sbh. report.json is
respelled into the fixture root, as ts1's spells /Users. In the fixture copies only, the hostname and the hash pins of the
qualifier, old binary, macOS facts and Pillow change. rows' tool_sha256 is set to the chain's literal, since the
fixture's bridge differs by one line (rehearse_stage07.py says why). The fake table frames are not real frames: this
proves the rules run and refuse, and it cannot show a real timeline meets them.

Takes are APFS clones, so they use no space and share no inode with production. Nothing is written in the repository,
and nothing is deleted. Run from the repository root on the local host:

    python3 -I -B <bundle>/checks/rehearse_chain03.py SCRATCH_FOLDER [FINISHED_STAGE07_REHEARSAL_ROOT]
"""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve()
BUNDLE = HERE.parents[1]
PIPE = BUNDLE.parent
REAL = HERE.parents[7]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


BASE = load('rehearse_n2', PIPE / 'chain02-n2/checks/rehearse_n2.py')
B2 = load('rehearse_build02', PIPE / 'qualify02/checks/rehearse_build02.py')
C, Q = BASE.C, BASE.Q
sha = Q.sha
FOLDER = C.CHAIN.replace('/chain01', '/chain03')
DERIVE = 'complex-remote-narration-derive02'
STAGES = ('derive', 'timeline', 'score', 'preview', 'pictures')
TIMELINE = 'complex-remote-timeline03/timeline-output'
TREE = Q.EC + '/native-stage07-root01'
CRATE = TREE + '/workspace/explainer_corpus/complex_analysis_visual'
NARRATION = Q.EC + '/production-audio-stage07-01/native-narration01'
ROWS = Q.EC + '/remote-ts1-01/stage07-rows01.json'
BRIDGE_SHA256 = (BUNDLE / 'ts1_chain03.py').read_text().split("BRIDGE_SHA256 = '")[1].split("'")[0]
STAGE07_REHEARSAL = PIPE.parent / 'complex-frederick-stage07-reuse01/checks/rehearse_stage07.py'

TUNE = r'''
def tune(out):
    """The kit's order and arithmetic: rows by kind, seconds and samples from frames, then stage07's one change."""
    import wave
    repaired = c.get('repaired')
    for chapter in c['chapters']:
        timeline, cues, log = (out / f'{chapter}.{table}.tsv' for table in ('timeline', 'cues', 'cuelog'))
        rows = [line.split('\t') for line in timeline.read_text().splitlines()]
        rows = [row for row in rows if row[0] != 'breath']
        rows += [['mark', 'predict', '1'], ['breath', '3', '0.05'], ['breath', '5', '0.05'], ['listen', '5', '0.50', f'{chapter[:2]}:end']]
        frames = {row[1]: int(row[2]) for row in rows if row[0] == 'line'}
        if repaired and chapter == '09_boundary':
            pivot = frames['09_5']
            later = lambda frame: frame + c['shift'] if frame > pivot else frame
            column = {'line': 2, 'mark': 2, 'breath': 1, 'listen': 1}
            for row in rows:
                if row[0] in column:
                    row[column[row[0]]] = str(later(int(row[column[row[0]]])))
                elif row[0] == 'frames':
                    row[1] = str(int(row[1]) + c['shift'])
            frames = {cue: later(frame) for cue, frame in frames.items()}
        if repaired and case == 'moved-chapter' and chapter == '05_gallery':
            next(row for row in rows if row[0] == 'frames')[1] = str(c['frames'] + 1)
        timeline.write_text('\n'.join('\t'.join(row) for row in rows) + '\n')
        cues.write_text(''.join(f'{cue}\t{frame / 60:.3f}\t1.000\n' for cue, frame in frames.items()))
        lines = []
        for line in log.read_text().splitlines():
            row = line.split('\t')
            if not line.startswith('#'):
                cue = Path(row[0]).stem
                row[3], row[12] = str(frames[cue]), str(frames[cue] * 800)
                if repaired and cue == '09_5':
                    data = Path(row[0]).read_bytes()
                    with wave.open(row[0]) as handle:
                        row[11] = str(handle.getnframes())
                    row[1:3] = [hashlib.sha256(data).hexdigest(), str(len(data))]
            lines.append('\t'.join(row))
        log.write_text('\n'.join(lines) + '\n')


'''
CHAPTER_FRAMES = ("[int(line.split('\\t')[1]) for line in (timeline / f'{chapter}.timeline.tsv').read_text().splitlines() "
                  "if line.startswith('frames\\t')][0]")
EDITS = [
    ("if args[0] == 'facts':\n", TUNE + "if args[0] == 'facts':\n"),
    ("elif args[0] == 'score':\n", "    tune(out)\nelif args[0] == 'score':\n"),
    ("        files = [(stem, 0, c['frames'] * 800)]\n", f"        frames = {CHAPTER_FRAMES}\n        files = [(stem, 0, frames * 800)]\n"),
    ("            offset = start / 48000 - c['frames'] / 60\n", "            offset = start / 48000 - frames / 60\n"),
    ("str(path.stat().st_size), str(c['frames']), '60', repr(offset),", "str(path.stat().st_size), str(frames), '60', repr(offset),"),
    ("        shutil.copy(here / 'fake-tiny.mp4', out / f'{chapter}.mp4')\n",
     "        shutil.copy(here / ('fake-tiny.mp4' if frames == c['frames'] else 'fake-tiny-09.mp4'), out / f'{chapter}.mp4')\n"),
]
for old, new in EDITS:
    assert C.FAKE.count(old) == 1, old
    C.FAKE = C.FAKE.replace(old, new)


def audio_root():
    """A finished rehearse_stage07.py root: named by STAGE07_REHEARSAL_ROOT, or made here once, in about two minutes."""
    if not os.environ.get('STAGE07_REHEARSAL_ROOT'):
        scratch = Path(tempfile.mkdtemp(prefix='stage07-audio-')) / 'rehearsal'
        subprocess.run([sys.executable, '-I', '-B', str(STAGE07_REHEARSAL), str(scratch)], cwd=REAL, check=True, stdout=subprocess.DEVNULL)
        os.environ['STAGE07_REHEARSAL_ROOT'] = str(scratch / 'root')
    return Path(os.environ['STAGE07_REHEARSAL_ROOT'])


class Real(BASE.Real):
    """After finish, script is chain03's fixture copy. stage('timeline') keeps its old meaning, chain01's refusal, and then
    also runs what stands before chain03, so start2 and text2 mean chain03 from there on."""
    shift = 8  # whole frames the longer 09_5 adds. 0 keeps every chapter's length.
    stage07 = False

    def rows(self):
        super().rows()
        B2.extend_rows(self)

    def config(self, case):
        super().config(case)
        if self.stage07:
            path = self.O / 'fake-config.json'
            data = json.loads(path.read_text())
            data.update(repaired=True, shift=self.shift, timeline_out=str(self.J / TIMELINE))
            path.write_text(json.dumps(data))

    def finish(self):
        super().finish()
        B2.add_build(self)
        folder = self.R / FOLDER
        folder.mkdir(exist_ok=True)
        shutil.copy(BUNDLE / 'start-chain03.sh', folder)
        text = (BUNDLE / 'ts1_chain03.py').read_text()
        for name, value in (('QUALIFIER_SHA256', sha(self.qualify02)), ('BASE_BIN_SHA256', sha(self.O / 'complex_analysis_visual-release01'))):
            text = text.replace(text.split(f"{name} = '")[1].split("'")[0], value)
        import PIL
        self.script = folder / 'ts1_chain03.py'
        self.script.write_text(text.replace(text.split("PILLOW = '")[1].split("'")[0], PIL.__version__))

    def stage(self, name):
        first = super().stage(name)
        for stage in STAGES:
            run = BASE.Real.start2(self, stage)
            assert run.returncode == 0, BASE.Real.text2(self, stage)[-600:]
        self.install_stage07()
        run = B2.start_build(self)
        assert run.returncode == 0, (self.O / 'build-controller02.log').read_text()[-600:]
        return first

    def install_stage07(self):
        root = audio_root()
        rows = json.loads((root / ROWS).read_text())
        for row in rows['files']:
            source, target = root / row['path'], self.R / row['path']
            if not target.exists():
                target.parent.mkdir(parents=True, exist_ok=True)
                if target.suffix == '.wav':
                    subprocess.run(['/bin/cp', '-c', str(source), str(target)], check=True)
                else:
                    shutil.copyfile(source, target)
        report = (root / NARRATION / 'report.json').read_bytes()
        spelled = json.loads(report)['lines']['09_5'][0]['source'].split('/narration_lab/')[0]
        fixed = report.replace(spelled.encode(), str(self.R).encode())
        (self.R / NARRATION / 'report.json').write_bytes(fixed)
        assembly = (root / NARRATION / 'assembly.json').read_bytes().replace(sha(root / NARRATION / 'report.json').encode(), sha(self.R / NARRATION / 'report.json').encode())
        (self.R / NARRATION / 'assembly.json').write_bytes(assembly)
        shutil.copyfile(self.C / 'oracle.py', self.R / CRATE / 'oracle.py')
        receipt = json.loads((self.R / TREE / 'tree-receipt.json').read_text())
        for row in receipt['files']:
            if row['path'].endswith('/oracle.py'):
                row['sha256'] = sha(self.R / row['path'])
        (self.R / TREE / 'tree-receipt.json').write_text(json.dumps(receipt))
        faked = {f'{NARRATION}/report.json', f'{NARRATION}/assembly.json', f'{CRATE}/oracle.py', f'{TREE}/tree-receipt.json'}
        for row in rows['files']:
            found = sha(self.R / row['path'])
            assert row['path'] in faked or found == row['sha256'], row['path']
            row['sha256'] = found
        rows['tool_sha256'] = BRIDGE_SHA256
        (self.R / ROWS).write_text(json.dumps(rows))
        self.N, self.workspace, self.C = self.R / NARRATION, self.R / TREE / 'workspace', self.R / CRATE
        self.stage07 = True
        self.config(self.case)
        frames = C.FRAMES + self.shift
        subprocess.run(['ffmpeg', '-v', 'error', '-nostdin', '-f', 'lavfi', '-i', 'color=c=black:s=960x540:r=60', '-f', 'lavfi', '-i', 'anullsrc=r=48000:cl=stereo',
                        '-frames:v', str(frames), '-t', str(frames / 60), '-c:v', 'libx264', '-preset', 'ultrafast', '-pix_fmt', 'yuv420p', str(self.O / 'fake-tiny-09.mp4')], check=True)

    def start2(self, stage):
        return subprocess.run(['bash', str(self.R / FOLDER / 'start-chain03.sh'), stage], env=self.env, capture_output=True, text=True)

    def text2(self, stage):
        job = DERIVE if stage == 'derive' else f'complex-remote-{stage}03'
        logs = [self.O / f'chain-{stage}-controller03.log', self.J / job / 'command.log']
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


def read(path):
    return json.loads(path.read_text())


def edit_rows(key, change):
    def damage(t):
        path = t.R / ROWS
        data = read(path)
        change(data)
        path.write_text(json.dumps(data))
    return path_of(ROWS), damage


def path_of(relative):
    return lambda t: t.R / relative


def set_tool(data):
    data['tool_sha256'] = '0' * 64


def set_script(data):
    data['script_sha256'] = '0' * 64


def set_cue(data):
    data['comparison']['cue_changed'] = '09_4'


def appended(path_getter):
    def damage(t):
        with path_getter(t).open('a') as handle:
            handle.write('# edit\n')
    return path_getter, damage


def changed_wav(t):
    return t.N / '09_6.wav'


def flipped(t):
    BASE.flip(changed_wav(t))


def baseline_table(t):
    return t.J / 'complex-remote-timeline02/timeline-output/01_the_bump.timeline.tsv'


def qualifier(t):
    return t.qualify02


REFUSALS = {
    'baseline-table-edited': (*appended(baseline_table), 'timeline02 table changed: 01_the_bump'),
    'rows-from-another-tool': (*edit_rows('tool', set_tool), 'Audio rows came from another tool'),
    'rows-bind-another-script': (*edit_rows('script', set_script), 'Audio rows bind another script'),
    'rows-name-another-cue': (*edit_rows('cue', set_cue), 'Audio rows differ'),
    'qualifier-edited': (*appended(qualifier), 'Sealed qualifier changed'),
    'n1-take-edited': (changed_wav, flipped, '09_6.wav'),
}
BOUNDARY_PLANTS = {
    'later-row-one-frame-off': ('timeline', 'line\t09_7\t13\t', 'line\t09_7\t14\t', 'is not'),
    'earlier-row-moved': ('timeline', 'line\t09_2\t1\t', 'line\t09_2\t2\t', 'comes before 09_5'),
    'repaired-cue-moved': ('timeline', 'line\t09_5\t4\t', 'line\t09_5\t5\t', '09_5 itself moved'),
    'earlier-mark-moved': ('timeline', 'mark\tpredict\t1\n', 'mark\tpredict\t9\n', 'comes before 09_5'),
    'later-hold-left-behind': ('timeline', 'breath\t13\t0.05', 'breath\t5\t0.05', 'is not'),
    'frames-header-not-moved': ('timeline', 'frames\t14', 'frames\t6', 'is not'),
    'later-row-other-gain': ('timeline', 'line\t09_6\t13\t0\t', 'line\t09_6\t13\t1\t', 'is not'),
    'cue-seconds-by-subtraction': ('cues', '09_6\t0.217\t', '09_6\t0.216\t', 'is not 09_6 at 0.217'),
    'cue-length-changed': ('cues', '09_7\t0.217\t1.000', '09_7\t0.217\t1.001', 'with length'),
    'later-sample-off-by-one': ('cuelog', f'\t{13 * 800}\n', f'\t{13 * 800 + 1}\n', 'cuelog'),
}


def table(t, kind, chapter='09_boundary'):
    return t.J / TIMELINE / f'{chapter}.{kind}.tsv'


def compare_planted(t, path, old, new, chapter):
    """One edit in the finished job's table, the read-only compare, the bytes back."""
    saved = path.read_bytes()
    text = saved.decode()
    assert old in text, old
    path.write_text(text.replace(old, new, 1))
    run = t.mode2('--compare', chapter)
    path.write_bytes(saved)
    return run


def name_probe(t):
    return (f"import importlib.util\nspec = importlib.util.spec_from_file_location('c', {str(t.script)!r})\nm = importlib.util.module_from_spec(spec)\nspec.loader.exec_module(m)\n"
            "print(m.M.CHAPTERS == m.CHAPTERS, m.M.N == m.N, m.M.job('score').name, m.M.check_cue_log is m.check_cue_log)\n"
            "try:\n    m.M.nonsense\nexcept AttributeError:\n    print('refused')\n")


def main():
    assert len(sys.argv) in (2, 3), __doc__
    if len(sys.argv) == 3:
        os.environ['STAGE07_REHEARSAL_ROOT'] = sys.argv[2]
    root = Path(sys.argv[1]).resolve()
    root.mkdir()
    Q.FIXTURE[0] = str(root)
    shared = root / 'shared'
    shared.mkdir()
    subprocess.run(['ffmpeg', '-v', 'error', '-nostdin', '-f', 'lavfi', '-i', 'color=c=black:s=960x540:r=60', '-f', 'lavfi', '-i', 'anullsrc=r=48000:cl=stereo',
                    '-frames:v', str(C.FRAMES), '-t', '0.1', '-c:v', 'libx264', '-preset', 'ultrafast', '-pix_fmt', 'yuv420p', str(shared / 'tiny.mp4')], check=True)
    t, run = fresh(root, 'baseline', shared)
    old_bin = sha(t.O / 'complex_analysis_visual-release01')
    summary = read(t.J / 'complex-remote-timeline02/timeline-summary.json')
    C.check('sealed-chain02-made-the-baseline', run.returncode != 0 and summary['cues'] == 105 and summary['binary_sha256'] == old_bin and (t.O / 'chain-pictures-controller02.exit').read_text().strip() == '0',
            'chain01 refused, then the five sealed chain02 stages ran')
    digest = (t.J / 'complex-remote-build02/binary.sha256').read_text().strip()
    C.check('build02-ran-after-chain02', digest == sha(t.O / 'complex_analysis_visual-release02') != old_bin, 'through start-build02.sh with the real guard')
    before = {p.name: sha(p) for p in t.N.iterdir()}
    run = t.mode2('--check', 'derive')
    lines = run.stdout.strip().splitlines()
    dry = json.loads(lines[-1]) if run.returncode == 0 else {}
    C.check('check-derive-dry-run', run.returncode == 0 and dry['derivation'] == {'cues': 105, 'rows': 414, 'source_fields_changed': 414, 'other_fields_changed': 0, 'samefile': 414, 'take_hash_equal': 414, 'takes': 105},
            C.last(run.stderr, '105 cues, 414 rows, 414 source fields, 0 others, on the stage07 report'))
    C.check('dry-run-both-tools', run.returncode == 0 and dry['tools']['standalone_refuses_N'] and dry['tools']['delivery_cues'] == 105, 'stage07 N refused as spelled; its derivation passes both unmodified tools')
    C.check('check-spends-no-name', not (t.J / DERIVE).exists() and not (t.O / 'chain-derive-inputs03.json').exists())
    for name, (path, damage, words) in REFUSALS.items():
        saved = path(t).read_bytes()
        damage(t)
        run = t.mode2('--check', 'derive')
        path(t).write_bytes(saved)
        C.check('refuse-' + name, run.returncode != 0 and words in run.stderr, C.last(run.stderr, 'no output'))
    stray = t.N / 'stray.wav'
    stray.write_bytes(b'x')
    run = t.mode2('--check', 'derive')
    stray.rename(t.base / 'stray.moved')
    C.check('refuse-extra-n1-file', run.returncode != 0 and 'File set changed' in run.stderr, C.last(run.stderr, 'no output'))
    for stage in STAGES:
        run = t.start2(stage)
        job = DERIVE if stage == 'derive' else f'complex-remote-{stage}03'
        d = read(t.J / job / 'resource-receipt.json')
        C.check('stage-' + stage, run.returncode == 0 and d['exit'] == 0 and d['inputs_sha256'] == d['final_inputs_sha256'] and d['policy'] == Q.POLICY,
                C.last(t.text2(stage), 'no output') if run.returncode else 'the real guard admitted and closed the job')
    C.check('N1-untouched', before == {p.name: sha(p) for p in t.N.iterdir()}, '110 files')
    same = [p.name for p in t.N.iterdir() if sha(p) == sha(t.n2 / p.name)]
    C.check('108-of-110-identical', len(same) == 108 and {p.name for p in t.N.iterdir()} - set(same) == {'report.json', 'assembly.json'})
    s = read(t.J / DERIVE / 'derive-summary.json')
    C.check('derive-provenance', s['narration'] == str(t.n2) and s['derived_from'] == str(t.N) and s['files_identical'] == 108)
    compared = read(t.J / 'complex-remote-timeline03/baseline-table-comparison.json')
    cells = sum(row['path_cells_mapped'] for row in compared['tables'])
    C.check('fourteen-chapters-equal-timeline02', len(compared['tables']) == 42 and all(row['normalized_equal'] for row in compared['tables']) and cells == compared['expected_path_cells'] == 192,
            f"{len(compared['tables'])} tables, {cells} path cells, 96 cues")
    b = compared['chapter09']
    C.check('chapter09-holds-the-rules', b['problems'] == [] and b['frame_shift'] == t.shift and b['repaired_frame'] == {'baseline': 4, 'new': 4} and b['path_cells_mapped'] == 18,
            f"9 cues, 18 path cells, later rows moved {b['frame_shift']} frames")
    old_cue = dict(line.split('\t')[:2] for line in table(t, 'cues').read_text().splitlines())['09_6']
    base_cue = dict(line.split('\t')[:2] for line in (t.J / 'complex-remote-timeline02/timeline-output/09_boundary.cues.tsv').read_text().splitlines())['09_6']
    C.check('cue-seconds-come-from-frames', old_cue == '0.217' and base_cue == '0.083' and f'{float(base_cue) + t.shift / 60:.3f}' == '0.216', '0.083 plus 8 frames would give 0.216; frame 13 is 0.217')
    row = next(line.split('\t') for line in table(t, 'cuelog').read_text().splitlines() if line.startswith(f'{t.n2}/09_5.wav'))
    C.check('09_5-row-is-the-new-take', row[1] == sha(t.n2 / '09_5.wav') and row[11] == '432000', '18.00 s, 432000 samples at 24 kHz')
    timeline = read(t.J / 'complex-remote-timeline03/timeline-summary.json')
    C.check('full-equivalence-not-claimed', timeline['full_baseline_equivalence_established'] is False and timeline['chapter09_frame_shift'] == t.shift)
    C.check('tables-spell-N2', f'{t.n2}/01_1.wav' in table(t, 'timeline', '01_the_bump').read_text() and str(t.N) not in table(t, 'timeline', '01_the_bump').read_text())
    for chapter in ('01_the_bump', '09_boundary'):
        run = t.mode2('--compare', chapter)
        C.check('compare-' + chapter, run.returncode == 0, C.last(run.stderr, 'read-only, exit 0'))
    for name, (kind, old, new, words) in BOUNDARY_PLANTS.items():
        run = compare_planted(t, table(t, kind), old, new, '09_boundary')
        C.check('stop-' + name, run.returncode == 1 and words in json.loads(run.stdout)['problems'][0] if run.returncode == 1 else False, C.last(run.stderr, json.loads(run.stdout)['problems'][0][:80] if run.stdout.strip() else 'no output'))
    run = compare_planted(t, table(t, 'cuelog'), sha(t.n2 / '09_5.wav'), '0' * 64, '09_boundary')
    C.check('stop-09_5-hash-wrong', run.returncode == 1 and 'cuelog' in run.stdout)
    run = compare_planted(t, table(t, 'timeline'), f'{t.n2}/09_6.wav', f'{t.n2.resolve()}/09_6.wav', '09_boundary')
    C.check('stop-physical-path-spelling', run.returncode == 1 and 'timeline' in run.stdout)
    run = compare_planted(t, table(t, 'timeline', '01_the_bump'), 'line\t01_1\t0\t', 'line\t01_1\t1\t', '01_the_bump')
    C.check('stop-unaffected-chapter-moved', run.returncode == 1 and '"normalized_equal": false' in run.stdout)
    manifests = {stage: {row['path']: row['sha256'] for row in read(t.O / f'chain-{stage}-inputs03.json')['files']} for stage in STAGES}
    C.check('release02-in-every-manifest', all(rows.get(str(t.O / 'complex_analysis_visual-release02')) == digest for rows in manifests.values()), 'frozen by hash from derive on, so the guard re-hashes it at each job end')
    last = manifests['pictures']
    wanted = [t.N / 'report.json', t.n2 / 'report.json', t.R / ROWS, t.script, t.qualify02, t.J / 'complex-remote-build02/build-summary.json', t.J / 'complex-remote-timeline02/timeline-summary.json',
              t.O / 'chain-timeline-controller02.exit', t.J / 'complex-remote-timeline03/baseline-table-comparison.json', t.J / 'complex-remote-preview03/preview-summary.json']
    C.check('last-manifest-carries-the-chain', all(str(p) in last for p in wanted) and not [p for p in last if p.startswith(str(t.physical))], 'N1, N2, rows, helpers, build, baseline and every stage, all by logical path')
    first = read(t.J / 'complex-remote-pictures03/resource-receipt.json')['command'][0]
    C.check('pictures-ran-under-its-own-python', first != read(t.J / 'complex-remote-build02/resource-receipt.json')['command'][0], 'and its build check passed there')
    C.check('pictures-not-viewed', read(t.J / 'complex-remote-pictures03/pictures/extraction.json')['pictures_viewed'] is False)
    C.check('old-binary-preserved', sha(t.O / 'complex_analysis_visual-release01') == old_bin)
    run = subprocess.run([t.python, '-I', '-B', '-c', name_probe(t)], cwd=t.R, env=t.env, capture_output=True, text=True)
    C.check('names-for-the-helpers', run.stdout.split() == ['True', 'True', 'complex-remote-score03', 'True', 'refused'], C.last(run.stderr, 'L.M resolves here; an unknown name is an AttributeError'))
    run = t.start2('timeline')
    C.check('second-start-refused', run.returncode == 1 and 'refused' in run.stderr)
    run = subprocess.run(['bash', str(t.R / FOLDER / 'start-chain03.sh'), 'nonsense'], env=t.env, capture_output=True, text=True)
    C.check('unknown-stage-refused', run.returncode == 2 and 'usage' in run.stderr)
    t, _ = fresh(root, 'moved-chapter', shared, 'moved-chapter')
    assert t.start2('derive').returncode == 0, t.text2('derive')
    run = t.start2('timeline')
    text = t.text2('timeline')
    evidence = t.J / 'complex-remote-timeline03/baseline-table-comparison.json'
    C.check('stop-a-chapter-that-moved', run.returncode != 0 and 'ts1 tables differ from timeline02' in text and evidence.exists() and not (t.J / 'complex-remote-timeline03/timeline-summary.json').exists(),
            C.last(text, 'no output'))
    print(f"RESULT {'PASS' if all(C.results) else 'FAIL'}, {C.results.count(False)} failed")
    return 0 if all(C.results) else 1


if __name__ == '__main__':
    sys.exit(main())
