#!/usr/bin/env python3
"""Tests for apportionment-csd-master735.py, run beside it:  python3 -I -B apportionment-csd-master735-tests.py [--actual DIR]

DIR holds root's copies of master700's actual apportionment.en.srt and apportionment.en.vtt (record 734). They are only
read and copied into a scratch folder. Without --actual, the tests that need them are skipped and the run says so.

Actual here: the source of master700 and master735, the 727 helper, verify701's parser and fast-caption line, root's record
734 and, with --actual, master700's caption bytes. Synthetic here: every job folder, final/, the movie bytes, the chapter
06 manifest and job, the reference tables and the support671 stand-in. Nothing here runs the CSD job, the guard, support671
or the native binary; that is production, and it has not run. Scratch files are left in place; nothing here deletes.
"""
import ast
import hashlib
import importlib.util
import json
import os
import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.dont_write_bytecode = True

ACTUAL = None
if '--actual' in sys.argv:
    at = sys.argv.index('--actual')
    ACTUAL = Path(sys.argv[at + 1]).resolve()
    del sys.argv[at:at + 2]

HERE = Path(__file__).resolve().parent
M700, M735 = HERE/'apportionment-csd-master700.py', HERE/'apportionment-csd-master735.py'
VERIFIER = HERE/'apportionment-csd-verify701.py'
HELPER727 = HERE/'apportionment-caption-repair727'/'repair_caption168.py'
RECORD734 = HERE/'root-caption-actual734.json'
PINS = {M700: 'e589f7f3302f9d97a9ad5b6b6f244ea3fccac71ffb91f5364f83fe555575743d',
        VERIFIER: '02ef7a5f4aae336b4a3795b066352a85c9a536d953c8920fab4755a2f174417f',
        HELPER727: 'd94aab217a6ee6639328414b774a7a374fb610c0716f3213222ca421a3762b9f',
        RECORD734: '3659f8fa12f535e5adbbe37d69f23110e27f9df6aeff66ec7a8bd395aee83324'}
FAST_LINE = (r"fast = [dict(index=i+1, **c) for i, c in enumerate(caps) "
             r"if len(c['text'].replace('\n', ' '))*1000/(c['end_ms']-c['start_ms']) > 20]")
FILM_FRAMES = 100886
NAMES = ('apportionment.en.srt', 'apportionment.en.vtt')
CH06 = '06_not_every_time'
REGISTRY = ['01_a', '02_b', '03_c', '04_d', '05_e', CH06] + [f'{k:02}_x' for k in range(7, 16)]


def sha_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha(path):
    return sha_bytes(Path(path).read_bytes())


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


master = load('master735', M735)
verifier = load('verify701', VERIFIER)
SCRATCH = Path(tempfile.mkdtemp(prefix='master735-'))


def real_fast(caps):
    namespace = {'caps': caps}
    exec(FAST_LINE, namespace)
    return namespace['fast']


def top(path):
    functions, names, imports, main = {}, {}, [], None
    for node in ast.parse(path.read_text()).body:
        if isinstance(node, ast.FunctionDef):
            functions[node.name] = node
        elif isinstance(node, ast.Assign):
            names[ast.unparse(node.targets[0])] = ast.dump(node.value)
        elif isinstance(node, (ast.Import, ast.ImportFrom)):
            imports.append(ast.dump(node))
        elif isinstance(node, ast.If):
            main = node
    return functions, names, imports, main


def lines(node):
    return [x.strip() for x in ast.unparse(node).splitlines()]


class Case(unittest.TestCase):
    def setUp(self):
        self.dir = Path(tempfile.mkdtemp(prefix='t', dir=SCRATCH))


class Pins(unittest.TestCase):
    """Actual files: the reviewed inputs this successor relies on."""

    def test_reviewed_files_are_the_pinned_ones(self):
        for path, digest in PINS.items():
            self.assertEqual(sha(path), digest, path.name)

    def test_caption_pins_are_record_734(self):
        record = json.loads(RECORD734.read_text())
        self.assertEqual(master.GENERATED_CAPTIONS, (record['input_srt_sha256'], record['input_vtt_sha256']))
        self.assertEqual(master.REPAIRED_CAPTIONS, (record['candidate_srt_sha256'], record['candidate_vtt_sha256']))
        self.assertEqual(master.CAPTION_HELPER_SHA, record['helper_sha256'])
        self.assertEqual(master.CAPTION_HELPER_SHA, sha(HELPER727))
        self.assertEqual((record['cues'], record['changed_srt_bytes'], record['changed_vtt_bytes'], record['fast_after']), (443, 2, 2, []))

    def test_the_helper_is_staged_at_the_same_place_under_q(self):
        self.assertEqual(master.CAPTION_HELPER.relative_to(master.Q), HELPER727.relative_to(HERE))
        self.assertEqual(master.CAPTION_NAMES, NAMES)

    def test_the_verifier_threshold_line_is_unchanged(self):
        self.assertIn(FAST_LINE, [x.strip() for x in VERIFIER.read_text().splitlines()])

    def test_chapter_06_is_unbound_and_refused(self):
        self.assertIsNone(master.CHAPTER06_MANIFEST)
        self.assertIsNone(master.CHAPTER06_MANIFEST_SHA256)
        with self.assertRaisesRegex(AssertionError, 'chapter 06 is not bound'):
            master.require_chapter06()


class SameAs700(unittest.TestCase):
    """Actual source: everything not named here is master700's, statement for statement."""

    CHANGED = {'check_tables', 'survey', 'freeze', 'run'}
    NEW = {'require_chapter06', 'chapter06_binding', 'successor_chapter', 'caption_helper', 'repair_captions', 'finish'}
    CHANGED_NAMES = {'JOB', 'INPUTS', 'SCOPE'}
    NEW_NAMES = {'CHAPTER06_MANIFEST', 'CHAPTER06_MANIFEST_SHA256', 'CHAPTER06_KEYS', 'CAPTION_HELPER', 'CAPTION_HELPER_SHA',
                 'CAPTION_NAMES', 'GENERATED_CAPTIONS', 'REPAIRED_CAPTIONS', 'CAPTION_SCOPE'}
    REPLACED = {'survey': ['records += [csd_chapter('],
                'freeze': ["for path in (HELPER, SUPPORT, D / 'guard_csd02.py'", "print('INPUTS700'"],
                'run': ['materialized700.json', "out / 'apportionment.en.srt'", 'master700-summary.json', 'json.dump(dict(frames=TOTAL_FRAMES',
                        'MASTER700_COMPLETED'],
                'check_tables': []}

    @classmethod
    def setUpClass(cls):
        cls.old, cls.new = top(M700), top(M735)

    def test_every_other_function_is_identical(self):
        old, new = self.old[0], self.new[0]
        self.assertEqual(set(new) - set(old), self.NEW)
        self.assertEqual(set(old) - set(new), set())
        for name in set(old) - self.CHANGED:
            self.assertEqual(ast.dump(old[name]), ast.dump(new[name]), name)

    def test_every_other_constant_and_import_is_identical(self):
        old, new = self.old[1], self.new[1]
        self.assertEqual(set(new) - set(old), self.NEW_NAMES)
        for name in set(old) - self.CHANGED_NAMES:
            self.assertEqual(old[name], new[name], name)
        self.assertEqual(self.old[2], self.new[2])
        self.assertEqual(ast.unparse(self.new[3]), ast.unparse(self.old[3]).replace('master700', 'master735'))

    def test_changed_functions_keep_every_other_700_statement_in_order(self):
        for name, fragments in self.REPLACED.items():
            old, new = lines(self.old[0][name]), lines(self.new[0][name])
            replaced = [x for x in old if any(f in x for f in fragments)]
            self.assertEqual(len(replaced), len(fragments), (name, replaced))
            rest = iter(new)
            for line in old:
                if line not in replaced:
                    self.assertTrue(any(line == x for x in rest), (name, line))

    def test_the_replaced_statements_are_the_intended_ones(self):
        old, new = self.old[0], self.new[0]
        survey = lines(new['survey'])
        self.assertIn('binding = chapter06_binding(registry)', survey)
        self.assertIn("records += [successor_chapter(binding, comp, version, tags, o) if c == binding['chapter'] else "
                      "csd_chapter(c, comp, version, tags, o) for c in registry[2:]]", survey)
        for name, fragment, change in (('freeze', 'for path in (HELPER, SUPPORT, ', ('SUPPORT, ', 'SUPPORT, CAPTION_HELPER, ')),
                                       ('freeze', "print('INPUTS7", ('INPUTS700', 'INPUTS735')),
                                       ('run', 'materialized7', ('materialized700', 'materialized735'))):
            before = [x for x in lines(old[name]) if fragment in x]
            after = [x for x in lines(new[name]) if fragment in x]
            self.assertEqual(len(before), 1)
            self.assertEqual(after, [before[0].replace(*change, 1)])

    def test_the_summary_keeps_every_700_field_computed_the_same_way(self):
        def fields(fn):
            calls = [n for n in ast.walk(fn) if isinstance(n, ast.Call) and ast.unparse(n.func) == 'json.dump'
                     and 'srt_sha256=' in ast.unparse(n)]
            self.assertEqual(len(calls), 1)
            call = calls[0]
            return {k.arg: ast.unparse(k.value) for k in call.args[0].keywords}
        old, new = fields(self.old[0]['run']), fields(self.new[0]['finish'])
        self.assertEqual({k: new[k] for k in old}, old)
        self.assertEqual(set(new) - set(old), {'caption_repair', 'chapter06_manifest', 'chapter06_manifest_sha256'})

    def test_order_gate_first_repair_after_captions_and_before_hashes(self):
        run, fin = lines(self.new[0]['run']), lines(self.new[0]['finish'])
        at = {key: next(i for i, x in enumerate(run) if key in x) for key in
              ('require_chapter06()', 'o = load()', 'caption_helper()', "'captions', 'all'", 'finish(registry')}
        self.assertLess(at['require_chapter06()'], at['o = load()'])
        self.assertEqual(run[at['caption_helper()'] - 1], 'recheck(bound)')
        self.assertLess(at['caption_helper()'], at["'captions', 'all'"])
        self.assertLess(at["'captions', 'all'"], at['finish(registry'])
        self.assertEqual(run[at['finish(registry'] - 1], 'recheck(bound)')
        self.assertLess(next(i for i, x in enumerate(fin) if x.startswith('captions = repair_captions(out, JOB)')),
                        next(i for i, x in enumerate(fin) if 'master735-summary.json' in x))
        freeze = lines(self.new[0]['freeze'])
        self.assertLess(freeze.index('require_chapter06()'), freeze.index('o = load()'))

    def test_four_cpus_and_four_rayon_threads_stay(self):
        run = lines(self.new[0]['run'])
        self.assertIn('os.sched_setaffinity(0, {0, 1, 2, 3})', run)
        self.assertIn("os.environ.update(PATH='/usr/bin:/bin', RAYON_NUM_THREADS='4')", run)


class Synthetic06(Case):
    """Synthetic: a scratch D/Q/jobs tree, a manifest, a successor job and reference tables."""

    def setUp(self):
        super().setUp()
        self.d = self.dir/'d'
        self.q, self.jobs, self.t = self.d/'q', self.d/'jobs', self.d/'jobs'/'timeline'
        for p in (self.q, self.jobs, self.t/'timeline'):
            p.mkdir(parents=True)
        patcher = mock.patch.multiple(master, D=self.d, Q=self.q, JOBS=self.jobs, T=self.t)
        patcher.start()
        self.addCleanup(patcher.stop)
        self.job = self.jobs/'csd-apportionment-successor-06'
        self.manifest(dict(chapter=CH06, job=str(self.job), script=str(self.q/'final-successor.py'),
                           inputs=str(self.q/'final-successor-06-inputs.json'), summary='successor-summary.json',
                           layout_sha256='a' * 64))

    def manifest(self, data):
        path = self.q/'chapter06-reviewed.json'
        path.write_text(json.dumps(data))
        for name, value in (('CHAPTER06_MANIFEST', path), ('CHAPTER06_MANIFEST_SHA256', sha(path))):
            patcher = mock.patch.object(master, name, value)
            patcher.start()
            self.addCleanup(patcher.stop)
        return path


class Binding(Synthetic06):
    def refused(self, change, pattern):
        data = json.loads((self.q/'chapter06-reviewed.json').read_text())
        change(data)
        self.manifest(data)
        with self.assertRaisesRegex(AssertionError, pattern):
            master.chapter06_binding(REGISTRY)

    def test_a_good_manifest(self):
        b = master.chapter06_binding(REGISTRY)
        self.assertEqual((b['chapter'], b['job'], b['summary'], b['layout_sha256']), (CH06, self.job, 'successor-summary.json', 'a' * 64))

    def test_a_changed_manifest(self):
        with mock.patch.object(master, 'CHAPTER06_MANIFEST_SHA256', '0' * 64):
            with self.assertRaisesRegex(AssertionError, 'manifest changed'):
                master.chapter06_binding(REGISTRY)

    def test_a_manifest_outside_q(self):
        other = self.dir/'elsewhere.json'
        other.write_bytes((self.q/'chapter06-reviewed.json').read_bytes())
        with mock.patch.object(master, 'CHAPTER06_MANIFEST', other):
            with self.assertRaisesRegex(AssertionError, 'manifest changed'):
                master.chapter06_binding(REGISTRY)

    def test_unbound(self):
        with mock.patch.object(master, 'CHAPTER06_MANIFEST', None):
            with self.assertRaisesRegex(AssertionError, 'not bound'):
                master.chapter06_binding(REGISTRY)

    def test_keys(self):
        self.refused(lambda d: d.update(extra=1), 'extra')
        self.refused(lambda d: d.pop('extra') and d.pop('layout_sha256'), 'chapter')

    def test_another_chapter(self):
        self.refused(lambda d: d.update(chapter='07_x'), '07_x')

    def test_the_old_final672_job(self):
        self.refused(lambda d: d.update(job=str(self.jobs/('csd-apportionment-final672-'+CH06))), 'not a successor job')

    def test_paths_outside_their_folders(self):
        self.refused(lambda d: d.update(job=str(self.dir/'job')), 'not a successor job')
        self.refused(lambda d: d.update(job=str(self.job), script=str(self.dir/'s.py')), 's.py')
        self.refused(lambda d: d.update(script=str(self.q/'s.py'), inputs=str(self.dir/'i.json')), 'i.json')

    def test_bad_summary_or_layout(self):
        self.refused(lambda d: d.update(summary='../s.json'), 'summary')
        self.refused(lambda d: d.update(summary='s.json', layout_sha256='A' * 64), 'layout_sha256')


class Stub:
    def __init__(self):
        self.calls = []

    def success(self, path):
        assert Path(path).is_file(), path
        self.calls.append(Path(path))


class Successor(Synthetic06):
    VERSION, TAGS = 'ffmpeg version synthetic', ('Lavc1.2.3 libx264', 'Lavf4.5.6')

    def setUp(self):
        super().setUp()
        render = self.job/'render'
        (render/'soundtrack').mkdir(parents=True)
        self.files = {CH06+'.mp4': b'synthetic movie', 'soundtrack/take.wav': b'synthetic wav', CH06+'.layout.tsv': b'new layout\n'}
        for rel, data in self.files.items():
            (render/rel).write_bytes(data)
        inputs = self.q/'final-successor-06-inputs.json'
        inputs.write_text(json.dumps(dict(files=[])))
        self.receipt = dict(command=[master.PYTHON, '-I', '-B', str(self.q/'final-successor.py'), CH06], policy=master.POLICY,
                            inputs_sha256=sha(inputs))
        self.summary = dict(chapter=CH06, ffmpeg_version=self.VERSION, encoder_tags=list(self.TAGS),
                            result=dict(folder=str((self.job/'render').relative_to(self.d)), frames=1000,
                                        files={k: sha_bytes(v) for k, v in self.files.items()}, encoder_settings='x264 synthetic'))
        self.comp = dict(chapters={CH06: dict(frames=1000)})
        self.write()

    def write(self):
        (self.job/'resource-receipt.json').write_text(json.dumps(self.receipt))
        (self.job/'successor-summary.json').write_text(json.dumps(self.summary))

    def record(self):
        o = Stub()
        rec = master.successor_chapter(master.chapter06_binding(REGISTRY), self.comp, self.VERSION, self.TAGS, o)
        return rec, o

    def refused(self, change, pattern=''):
        change()
        self.write()
        with self.assertRaisesRegex(AssertionError, pattern):
            self.record()

    def test_a_good_successor(self):
        rec, o = self.record()
        self.assertEqual(o.calls, [self.job/'resource-receipt.json'])
        self.assertEqual((rec['chapter'], rec['imported'], rec['frames'], rec['layout_sha256']), (CH06, False, 1000, 'a' * 64))
        self.assertEqual(rec['records'], [self.job/'resource-receipt.json', self.job/'successor-summary.json',
                                          self.q/'final-successor-06-inputs.json', self.q/'chapter06-reviewed.json'])
        self.assertEqual(set(rec['files']), set(self.files))

    def test_another_command(self):
        self.refused(lambda: self.receipt['command'].__setitem__(3, str(self.q/'other.py')))

    def test_another_policy_or_inputs(self):
        self.refused(lambda: self.receipt.update(policy=dict(master.POLICY, work_cap_gib=57)))
        self.refused(lambda: self.receipt.update(policy=master.POLICY, inputs_sha256='0' * 64))

    def test_another_encoder(self):
        self.refused(lambda: self.summary.update(ffmpeg_version='other'))
        self.refused(lambda: self.summary.update(ffmpeg_version=self.VERSION, encoder_tags=['x', 'y']))

    def test_another_folder_or_frame_count(self):
        self.refused(lambda: self.summary['result'].update(folder='jobs/other/render'))
        self.refused(lambda: self.summary['result'].update(folder=str((self.job/'render').relative_to(self.d)), frames=999))

    def test_unsafe_or_missing_files(self):
        files = self.summary['result']['files']
        self.refused(lambda: files.update({'/abs.tsv': 'x'}), 'abs')
        self.refused(lambda: (files.pop('/abs.tsv'), files.update({'../up.tsv': 'x'})), 'up')
        self.refused(lambda: (files.pop('../up.tsv'), files.update({'missing.tsv': 'x'})), 'missing')

    def test_a_symlinked_file(self):
        files = self.summary['result']['files']
        os.symlink(self.job/'render'/(CH06+'.mp4'), self.job/'render'/'link.mp4')
        self.refused(lambda: files.update({'link.mp4': 'x'}), 'link')


class Tables06(Synthetic06):
    """check_tables on the repaired chapter: timeline and cues against timeline666, layout against the manifest."""

    def setUp(self):
        super().setUp()
        ref = self.t/'timeline'
        tables = {'timeline.tsv': b'chapter\t06\n', 'cues.tsv': b'cue\t06_1\n', 'cuelog.tsv': b'log\n', 'layout.tsv': b'old layout\n'}
        for ext, data in tables.items():
            (ref/(CH06+'.'+ext)).write_bytes(data)
        self.comp = dict(chapters={CH06: dict(layout_sha256=sha_bytes(tables['layout.tsv']),
                                               tables={e: sha_bytes(tables[e]) for e in ('timeline.tsv', 'cues.tsv', 'cuelog.tsv')})})
        self.render = self.job/'render'
        self.render.mkdir(parents=True)
        for ext in ('timeline.tsv', 'cues.tsv'):
            (self.render/(CH06+'.'+ext)).write_bytes(tables[ext])
        (self.render/(CH06+'.layout.tsv')).write_bytes(b'new layout\n')
        self.rec = dict(chapter=CH06, imported=False, layout_sha256=sha_bytes(b'new layout\n'),
                        files={CH06+'.'+e: self.render/(CH06+'.'+e) for e in ('timeline.tsv', 'cues.tsv', 'layout.tsv')})

    def test_same_timing_new_layout(self):
        master.check_tables(self.rec, self.comp, '/local/', None)

    def test_a_layout_other_than_the_manifest(self):
        with self.assertRaisesRegex(AssertionError, 'reviewed manifest'):
            master.check_tables(dict(self.rec, layout_sha256='b' * 64), self.comp, '/local/', None)

    def test_changed_timing(self):
        for ext in ('timeline.tsv', 'cues.tsv'):
            with self.subTest(ext=ext):
                path = self.render/(CH06+'.'+ext)
                data = path.read_bytes()
                path.write_bytes(data + b'extra\n')
                with self.assertRaisesRegex(AssertionError, 'differs from timeline666'):
                    master.check_tables(self.rec, self.comp, '/local/', None)
                path.write_bytes(data)

    def test_any_other_chapter_still_needs_timeline666_layout(self):
        plain = {k: v for k, v in self.rec.items() if k != 'layout_sha256'}
        with self.assertRaisesRegex(AssertionError, 'differs from timeline666'):
            master.check_tables(plain, self.comp, '/local/', None)


class CaptionRefusals(Case):
    """Synthetic caption bytes: the step refuses before it moves or writes anything."""

    def setUp(self):
        super().setUp()
        self.job, self.out = self.dir/'job', self.dir/'job'/'final'
        self.out.mkdir(parents=True)
        for name in NAMES:
            (self.out/name).write_bytes(b'1\n00:00:00,000 --> 00:00:01,000\nsynthetic\n\n')
        patcher = mock.patch.object(master, 'CAPTION_HELPER', HELPER727)
        patcher.start()
        self.addCleanup(patcher.stop)

    def untouched(self):
        self.assertEqual(sorted(p.name for p in self.job.iterdir()), ['final'])
        self.assertEqual(sorted(p.name for p in self.out.iterdir()), sorted(NAMES))

    def test_another_caption_pair(self):
        with self.assertRaisesRegex(AssertionError, 'another caption pair'):
            master.repair_captions(self.out, self.job)
        self.untouched()

    def test_a_changed_helper(self):
        changed = self.dir/'repair_caption168.py'
        changed.write_bytes(HELPER727.read_bytes() + b'\n')
        with mock.patch.object(master, 'CAPTION_HELPER', changed):
            with self.assertRaisesRegex(AssertionError, 'caption helper 727 changed'):
                master.repair_captions(self.out, self.job)
        self.untouched()

    def test_a_rebound_helper_with_other_values(self):
        other = self.dir/'repair_caption168.py'
        other.write_text(HELPER727.read_text().replace('NEW_END_MS = 616310', 'NEW_END_MS = 616330'))
        with mock.patch.multiple(master, CAPTION_HELPER=other, CAPTION_HELPER_SHA=sha(other)):
            with self.assertRaisesRegex(AssertionError, 'caption helper 727 values changed'):
                master.repair_captions(self.out, self.job)
        self.untouched()

    def test_existing_caption_folders(self):
        (self.job/'captions-repaired735').mkdir()
        with self.assertRaisesRegex(AssertionError, 'already exist'):
            master.repair_captions(self.out, self.job)

    def test_no_summary_when_the_repair_refuses(self):
        (self.dir/'inputs.json').write_text('{}')
        with mock.patch.multiple(master, JOB=self.job, INPUTS=self.dir/'inputs.json'):
            with self.assertRaisesRegex(AssertionError, 'another caption pair'):
                master.finish(REGISTRY, {}, {}, self.out)
        self.untouched()


@unittest.skipUnless(ACTUAL, 'no --actual DIR given: the actual-copy tests did not run')
class ActualCopies(Case):
    """master700's actual caption bytes (record 734), copied into scratch; the source copies are only read."""

    @classmethod
    def setUpClass(cls):
        cls.sources = [ACTUAL/n for n in NAMES]
        cls.before = tuple(sha(p) for p in cls.sources)
        assert cls.before == master.GENERATED_CAPTIONS, ('the --actual pair is not record 734 input', cls.before)

    @classmethod
    def tearDownClass(cls):
        assert tuple(sha(p) for p in cls.sources) == cls.before, 'an actual source copy changed'

    def setUp(self):
        super().setUp()
        self.job, self.out = self.dir/'job', self.dir/'job'/'final'
        self.out.mkdir(parents=True)
        for source in self.sources:
            shutil.copyfile(source, self.out/source.name)
        patcher = mock.patch.object(master, 'CAPTION_HELPER', HELPER727)
        patcher.start()
        self.addCleanup(patcher.stop)

    def test_actual_pair_transforms_and_the_original_bytes_survive(self):
        kit_files = [(self.out/n).stat().st_ino for n in NAMES]
        record = master.repair_captions(self.out, self.job)
        generated, repaired = self.job/'captions-generated735', self.job/'captions-repaired735'
        self.assertEqual([(generated/n).stat().st_ino for n in NAMES], kit_files, 'the kit files themselves are kept')
        for source, name in zip(self.sources, NAMES):
            self.assertEqual((generated/name).read_bytes(), source.read_bytes())
            self.assertTrue(os.path.samefile(self.out/name, repaired/name))
            old, new = (generated/name).read_bytes(), (self.out/name).read_bytes()
            self.assertEqual(len(old), len(new))
            self.assertEqual(sum(a != b for a, b in zip(old, new)), 2)
        self.assertEqual(tuple(sha(generated/n) for n in NAMES), master.GENERATED_CAPTIONS)
        self.assertEqual(tuple(sha(self.out/n) for n in NAMES), master.REPAIRED_CAPTIONS)
        self.assertEqual(sorted(p.name for p in self.out.iterdir()), sorted(NAMES))
        self.assertEqual(record['report']['cues'], 443)
        self.assertIn('Display dwell only', record['scope'])

        before = verifier.captions(generated/NAMES[0], FILM_FRAMES)
        after = verifier.captions(self.out/NAMES[0], FILM_FRAMES)
        self.assertEqual(before, verifier.captions(generated/NAMES[1], FILM_FRAMES, True))
        self.assertEqual(after, verifier.captions(self.out/NAMES[1], FILM_FRAMES, True))
        self.assertEqual(len(after), 443)
        self.assertEqual([x['index'] for x in real_fast(before)], [168])
        self.assertEqual(real_fast(after), [])
        self.assertEqual([i for i, (a, b) in enumerate(zip(before, after), 1) if a != b], [168])
        self.assertEqual((before[167]['end_ms'], after[167]['end_ms']), (616290, 616310))

    def test_synthetic_master_integration_of_the_actual_pair(self):
        """Synthetic job and movie around the actual caption bytes: finish() as run() calls it."""
        movie = self.out/'apportionment_master.mp4'
        movie.write_bytes(b'synthetic movie bytes, not a master')
        inputs, manifest = self.dir/'inputs.json', self.dir/'chapter06-reviewed.json'
        inputs.write_text('{}')
        manifest.write_text('{}')
        with mock.patch.multiple(master, JOB=self.job, INPUTS=inputs, CHAPTER06_MANIFEST=manifest,
                                 CHAPTER06_MANIFEST_SHA256=sha(manifest)):
            master.finish(REGISTRY, dict(host='synthetic'), dict(master='synthetic'), self.out)
            summary = json.loads((self.job/'master735-summary.json').read_text())
            self.assertTrue(sha(self.out/NAMES[0]) == summary['srt_sha256'] and sha(self.out/NAMES[1]) == summary['vtt_sha256'])
            self.assertEqual((summary['srt_sha256'], summary['vtt_sha256']), master.REPAIRED_CAPTIONS)
            self.assertEqual(summary['movie_sha256'], sha(movie))
            repair = summary['caption_repair']
            self.assertEqual((repair['generated']['srt_sha256'], repair['generated']['vtt_sha256']), master.GENERATED_CAPTIONS)
            self.assertEqual((repair['report']['input_srt_sha256'], repair['report']['output_srt_sha256']),
                             (master.GENERATED_CAPTIONS[0], master.REPAIRED_CAPTIONS[0]))
            self.assertEqual(summary['helper_sha256'], sha(M735))
            self.assertEqual(summary['chapter06_manifest_sha256'], sha(manifest))
            self.assertIn('display dwell', summary['scope'])
            with self.assertRaises(AssertionError):
                master.finish(REGISTRY, {}, {}, self.out)
        self.assertEqual(tuple(sha(self.job/'captions-generated735'/n) for n in NAMES), master.GENERATED_CAPTIONS)


if __name__ == '__main__':
    print('actual copies:', ACTUAL if ACTUAL else 'not given', file=sys.stderr)
    unittest.main()
