"""SYNTHETIC regression tests for exact ElevenLabs external-voice receipts.

Every WAV here is generated integer PCM written by this file. Nothing was
synthesised, heard, transcribed or rendered; a pass proves only the assembler's
receipt contract, never voice identity or listening acceptance.

Run through run_tests.py, which builds the two tool mirrors and passes them as
    python3 -I -B test_external_voice.py BASE_TOOLS_DIR PATCHED_TOOLS_DIR
BASE is the canonical tools copy (e0aa81f3...), PATCHED the bundle's copy.
"""
import copy
import importlib.util
import json
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import tempfile
import types
import unittest
import wave

BASE, PATCHED = (Path(a).resolve() for a in sys.argv[1:3])
FREDERICK = ('ElevenLabs UI, Frederick Surrey', 'Frederick Surrey - Smooth and Velvety')
OLIVER = ('ElevenLabs UI, Oliver', 'Oliver - Clean, British and Steady')
V2 = 'elevenlabs-ui-receipt-v2'
SCRIPT = {'01_1': 'Here is a smooth curve. It bends once.', '01_2': 'Now look closer.'}
FRAMES = {'01_1': 2400, '01_2': 1800}
ENDS = {'01_1': [1000, 2400], '01_2': [1800]}
GAPS = {'01_1': [0, 120], '01_2': [240]}


def load(tools, name, alias):
    spec = importlib.util.spec_from_file_location(alias, tools / name)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


base = load(BASE, 'coherent_narration.py', 'coherent_base')
m = load(PATCHED, 'coherent_narration.py', 'coherent_patched')
delivery = load(PATCHED, 'narration_delivery.py', 'delivery_patched')


def write_wav(path, frames, rate, salt):
    with wave.open(str(path), 'wb') as w:
        w.setparams((1, 2, rate, 0, 'NONE', 'not compressed'))
        w.writeframes(b''.join(struct.pack('<h', ((n + salt) * 37) % 2001 - 1000) for n in range(frames)))


def pcm(path):
    with wave.open(str(path), 'rb') as w:
        return w.readframes(w.getnframes())


class Fixture:
    """A miniature film: two one-cue units, decoded WAVs, downloads and requests (all synthetic)."""

    def __init__(self, root):
        self.root = root
        root.mkdir(parents=True)
        self.script = root / 'script.tsv'
        self.script.write_text(''.join(f'{c}\t{t}\n' for c, t in SCRIPT.items()))
        self.units = root / 'units.json'
        self.plan = {'schema_version': 1, 'script_sha256': m.cache.digest(self.script),
                     'units': [{'id': f'01u{i}', 'line_ids': [c], 'text': t}
                               for i, (c, t) in enumerate(SCRIPT.items(), 1)]}
        self.units.write_text(json.dumps(self.plan))
        self.wav, self.download, self.request = {}, {}, {}
        for i, cue in enumerate(SCRIPT):
            self.wav[cue] = root / f'{cue}-decoded24k.wav'
            write_wav(self.wav[cue], FRAMES[cue], 24000, 11 * i)
            self.download[cue] = root / f'{cue}-download01.wav'
            write_wav(self.download[cue], 2 * FRAMES[cue], 48000, 5 * i)

    def write_request(self, cue, voice, change=None):
        path = self.root / f'{cue}-request01.json'
        body = {'at': 'synthetic', 'cue': cue, 'text': SCRIPT[cue], 'text_sha256': 'synthetic',
                'voice': voice, 'model': 'Eleven v4', 'stability': 0.5, 'similarity': 0.75,
                'output': 'WAV 48 kHz Lossless', 'submitted': False,
                'limits': 'SYNTHETIC test fixture; no submission, no listening.'}
        body.update(change or {})
        path.write_text(json.dumps(body))
        self.request[cue] = path
        return path

    def row(self, cue, v2_voice=None):
        info = m.cache.wav_identity(self.wav[cue])
        row = {'cue': cue, 'text': SCRIPT[cue], 'wav': str(self.wav[cue]), 'wav_sha256': info['sha256'],
               'samples': info['frames'], 'rate': info['sample_rate'], 'source': str(self.download[cue]),
               'source_sha256': m.cache.digest(self.download[cue])}
        if v2_voice is not None:
            request = self.request.get(cue) or self.write_request(cue, v2_voice)
            row.update(voice=v2_voice, request=str(request), request_sha256=m.cache.digest(request))
        return row

    def receipt(self, label_voice, cues=tuple(SCRIPT), v2=True, name='external.json'):
        label, voice = label_voice
        body = {'source': label, 'provenance': 'SYNTHETIC fixture', 'records':
                [self.row(c, voice if v2 else None) for c in cues]}
        if v2:
            body = {'schema': V2, 'source': label, 'provider': 'ElevenLabs', 'voice': voice, **body}
        path = self.root / name
        path.write_text(json.dumps(body))
        return path, body

    def record(self, cue, receipt_path):
        unit = next(u for u in self.plan['units'] if u['line_ids'] == [cue])
        return {'id': unit['id'], 'wav': str(self.wav[cue]), 'sha256': m.cache.digest(self.wav[cue]),
                'generation_receipt': str(receipt_path), 'sentence_ends': ENDS[cue],
                'gap_after_samples': GAPS[cue]}

    def native_receipt(self, cue):
        """Same shape as tools/test_coherent_narration.py's native fixture; synthetic."""
        unit = next(u for u in self.plan['units'] if u['line_ids'] == [cue])
        info = m.cache.wav_identity(self.wav[cue])
        assets = {'talker_sha256': 'model', 'codec_sha256': 'codec', 'voice_source_sha256': 'voice'}
        for name in ('vocab.json', 'merges.txt', 'tokenizer_config.json'):
            assets[name] = {'sha256': name}
        generation = {'schema': 'ftts.generation', 'schema_version': 1, 'run_id': 'unit-fixture',
            'state': 'completed', 'execution': 'inline', 'requested': {'seed': 2},
            'effective': {'seed': 2, 'normalization': {'mode': 'verbatim'}, 'utterance_start': 'fresh',
                          'sampling': {'mode': 'production'}},
            'output': {'samples': info['frames'], 'frames': 10, 'frame_limit': 600, 'sample_rate': 24000,
                       'stop_reason': 'end_of_speech'}, 'producer': {'executable_sha256': 'exe'},
            'voice': {'route': 'xvector'}, 'assets': assets}
        context = {'settings': {'normalize': 'verbatim'}, 'executable': {'sha256': 'exe'},
            'voice': {'kind': 'file', 'input': {'sha256': 'voice'}}, 'model': '/synthetic/model.fttsq',
            'bundle': [{'path': '/synthetic/model.fttsq', 'sha256': 'model'},
                       {'path': '/synthetic/speech_tokenizer/model.safetensors', 'sha256': 'codec'}] +
                      [{'path': '/synthetic/' + n, 'sha256': n} for n in ('vocab.json', 'merges.txt', 'tokenizer_config.json')]}
        completion = {'run_id': 'unit-fixture', 'event': 'run_complete', 'exit_code': 0,
                      'samples': info['frames'], 'frames': 10}
        path = self.root / f'{cue}-native.json'
        path.write_text(json.dumps({'wav': info, 'returncode': 0, 'generation': generation,
                                    'completion': completion, 'request': {'seed': 2, 'context': context,
                                    'submitted_text': unit['text']}}))
        return path

    def spans(self, records, name='spans.json'):
        path = self.root / name
        path.write_text(json.dumps({'schema_version': 1, 'units_sha256': m.cache.digest(self.units),
            'boundary_review': {'reviewer': 'synthetic-test', 'basis': 'SYNTHETIC fixture; no listening',
                                'listening': 'unreviewed'}, 'units': records}))
        return path

    def args(self, spans, out):
        return types.SimpleNamespace(script=self.script, units=self.units, spans=spans,
                                     out=self.root / out, chapter='01')


class Base(unittest.TestCase):
    def setUp(self):
        self.fx = Fixture(Path(tempfile.mkdtemp(prefix='external-voice-')) / 'fx')

    def assembled_matches_independent_rebuild(self, out):
        """Rebuild each cue from the decoded sources with wave, not with the tool."""
        for cue in SCRIPT:
            source, start, expected = pcm(self.fx.wav[cue]), 0, b''
            for end, gap in zip(ENDS[cue], GAPS[cue]):
                expected += source[start * 2:end * 2] + b'\0\0' * gap
                start = end
            self.assertEqual(pcm(out / f'{cue}.wav'), expected, cue)


class PositiveTests(Base):
    def test_legacy_oliver_receipt_is_byte_identical_to_canonical(self):
        receipt, _ = self.fx.receipt(OLIVER, v2=False)
        spans = self.fx.spans([self.fx.record(c, receipt) for c in SCRIPT])
        base.assemble(self.fx.args(spans, 'out-base'))
        m.assemble(self.fx.args(spans, 'out-patched'))
        a, b = self.fx.root / 'out-base', self.fx.root / 'out-patched'
        self.assertEqual(sorted(p.name for p in a.iterdir()), sorted(p.name for p in b.iterdir()))
        for p in a.iterdir():
            self.assertEqual(p.read_bytes(), (b / p.name).read_bytes(), p.name)
        self.assertTrue(base.verify(b, self.fx.script, self.fx.units, chapter='01'))
        self.assertTrue(m.verify(a, self.fx.script, self.fx.units, chapter='01'))
        self.assembled_matches_independent_rebuild(b)

    def test_legacy_oliver_receipt_with_extra_helper_fields_still_accepted(self):
        # Shape written by logarithm-stage08-audio-preparation01: short voice, extra keys, no schema.
        receipt, body = self.fx.receipt(OLIVER, v2=False)
        body.update(voice='Oliver', provider='ElevenLabs', claim='synthetic')
        for row in body['records']:
            row.update(downloads_receipt='synthetic', source_pcm_sha256='synthetic')
        receipt.write_text(json.dumps(body))
        spans = self.fx.spans([self.fx.record(c, receipt) for c in SCRIPT])
        m.assemble(self.fx.args(spans, 'out'))
        self.assertTrue(m.verify(self.fx.root / 'out', self.fx.script, self.fx.units, chapter='01'))

    def test_frederick_v2_assembles_and_reconstructs_sample_for_sample(self):
        receipt, _ = self.fx.receipt(FREDERICK)
        spans = self.fx.spans([self.fx.record(c, receipt) for c in SCRIPT])
        out = self.fx.root / 'out'
        m.assemble(self.fx.args(spans, 'out'))
        self.assertTrue(m.verify(out, self.fx.script, self.fx.units, chapter='01'))
        self.assertTrue(m.verify(out))
        self.assembled_matches_independent_rebuild(out)
        report = json.loads((out / 'report.json').read_text())
        self.assertEqual([r['source'] for r in report['lines']['01_1']], [str(self.fx.wav['01_1'].resolve())] * 2)
        self.assertEqual(m.assembly_voice(json.loads(spans.read_text())['units']), FREDERICK[1])

    def test_frederick_v2_per_cue_receipts_also_accepted(self):
        records = []
        for cue in SCRIPT:
            receipt, _ = self.fx.receipt(FREDERICK, cues=(cue,), name=f'{cue}-external.json')
            records.append(self.fx.record(cue, receipt))
        m.assemble(self.fx.args(self.fx.spans(records), 'out'))
        self.assembled_matches_independent_rebuild(self.fx.root / 'out')

    def test_oliver_v2_and_legacy_oliver_share_one_voice(self):
        v2, _ = self.fx.receipt(OLIVER, cues=('01_1',), name='oliver-v2.json')
        legacy, _ = self.fx.receipt(OLIVER, cues=('01_2',), v2=False, name='oliver-legacy.json')
        spans = self.fx.spans([self.fx.record('01_1', v2), self.fx.record('01_2', legacy)])
        m.assemble(self.fx.args(spans, 'out'))
        self.assertTrue(m.verify(self.fx.root / 'out', self.fx.script, self.fx.units, chapter='01'))

    def test_canonical_base_fails_on_frederick_v2(self):
        # Documents the defect being repaired: the hardcoded label sends Frederick to the native branch.
        receipt, _ = self.fx.receipt(FREDERICK)
        with self.assertRaises(KeyError):
            base.unit_spans(self.fx.plan['units'][0], base.read_script(self.fx.script),
                            self.fx.record('01_1', receipt))


class NegativeTests(Base):
    def setUp(self):
        super().setUp()
        self.receipt, self.body = self.fx.receipt(FREDERICK)
        self.unit = self.fx.plan['units'][0]
        self.script = m.read_script(self.fx.script)
        self.rec = self.fx.record('01_1', self.receipt)
        m.unit_spans(self.unit, self.script, self.rec)  # the valid baseline passes

    def refuse(self, pattern, body=None, record=None, unit=None):
        if body is not None:
            self.receipt.write_text(json.dumps(body))
        try:
            with self.assertRaisesRegex(ValueError, pattern):
                m.unit_spans(unit or self.unit, self.script, record or self.rec)
        finally:
            self.receipt.write_text(json.dumps(self.body))

    def changed(self, edit):
        body = copy.deepcopy(self.body)
        edit(body)
        return body

    def row0(self, **change):
        return self.changed(lambda b: b['records'][0].update(change))

    def test_malformed(self):
        self.refuse('JSON object', body=[self.body])
        self.refuse('must state the receipt voice', body=self.changed(lambda b: b.pop('records')))
        self.refuse('must state the receipt voice', body=self.changed(lambda b: b.update(records=[])))
        self.refuse('must state the receipt voice', body=self.changed(lambda b: b['records'][1].pop('voice')))
        self.refuse('lacks request, request_sha256',
                    body=self.changed(lambda b: [b['records'][0].pop(k) for k in ('request', 'request_sha256')]))
        self.refuse('must each name a cue', body=self.changed(lambda b: b['records'][1].pop('cue')))
        self.refuse('exactly one source per cue',
                    body=self.changed(lambda b: b['records'].append(copy.deepcopy(b['records'][0]))))
        self.refuse('exactly one source per cue', body=self.row0(cue='01_9'))
        legacy, body = self.fx.receipt(OLIVER, v2=False, name='legacy.json')
        body['records'][0].pop('samples')
        legacy.write_text(json.dumps(body))
        with self.assertRaisesRegex(ValueError, 'lacks samples'):
            m.unit_spans(self.unit, self.script, self.fx.record('01_1', legacy))
        request = self.fx.request['01_1']
        request.write_text(json.dumps([json.loads(request.read_text())]))
        self.refuse('cue, text and voice exactly', body=self.row0(request_sha256=m.cache.digest(request)))

    def test_wrong_source_label(self):
        for label in ('ElevenLabs UI, Frederick', 'ElevenLabs UI, Frederick Surrey ', 'elevenlabs ui, frederick surrey',
                      'ElevenLabs UI, Frederick Surrey - Smooth and Velvety', 'ElevenLabs', ['ElevenLabs UI, Oliver']):
            with self.subTest(label=label):
                self.refuse('no exact known source label', body=self.changed(lambda b: b.update(source=label)))
        legacy_style = self.changed(lambda b: b.pop('schema'))
        self.refuse('unrecognised external UI receipt', body=legacy_style)
        self.refuse('unrecognised external UI receipt', body=self.changed(lambda b: b.update(schema='elevenlabs-ui-receipt-v3')))
        oliver_v3 = self.changed(lambda b: b.update(schema='elevenlabs-ui-receipt-v3', source=OLIVER[0]))
        self.refuse('unrecognised external UI receipt', body=oliver_v3)
        self.refuse('neither native FrankenTTS', body=self.changed(lambda b: [b.pop('schema'), b.update(source=FREDERICK[1])]))
        self.refuse('provider or voice', body=self.changed(lambda b: b.update(provider='ElevenLabs Inc.')))
        self.refuse('provider or voice', body=self.changed(lambda b: b.pop('provider')))

    def test_wrong_voice_no_substring_acceptance(self):
        for voice in ('Frederick Surrey', 'Frederick', OLIVER[1], FREDERICK[1] + ' ', 'frederick surrey - smooth and velvety'):
            with self.subTest(voice=voice):
                self.refuse('provider or voice', body=self.changed(lambda b: b.update(voice=voice)))
                self.refuse('must state the receipt voice', body=self.row0(voice=voice))
        self.refuse('provider or voice', body=self.changed(lambda b: b.update(source=OLIVER[0])))
        request = self.fx.request['01_1']
        original = request.read_text()
        for voice in (OLIVER[1], 'Frederick Surrey', 'ElevenLabs ' + FREDERICK[1]):
            with self.subTest(request_voice=voice):
                self.fx.write_request('01_1', voice)
                self.refuse('cue, text and voice exactly', body=self.row0(request_sha256=m.cache.digest(request)))
        request.write_text(original)

    def test_mixed_voice_assembly_refused(self):
        oliver, _ = self.fx.receipt(OLIVER, cues=('01_2',), v2=False, name='oliver.json')
        spans = self.fx.spans([self.rec, self.fx.record('01_2', oliver)], name='mixed.json')
        with self.assertRaisesRegex(ValueError, 'one narrator voice'):
            m.assemble(self.fx.args(spans, 'out-mixed'))
        self.assertFalse((self.fx.root / 'out-mixed').exists())
        native = self.fx.native_receipt('01_2')
        # A native receipt alone still passes the unchanged native branch...
        m.unit_spans(self.fx.plan['units'][1], self.script, self.fx.record('01_2', native))
        self.assertEqual(m.assembly_voice([self.fx.record('01_2', native)]), m.NATIVE_VOICE)
        # ...but cannot share an assembly with an external voice.
        spans = self.fx.spans([self.rec, self.fx.record('01_2', native)], name='native-mixed.json')
        with self.assertRaisesRegex(ValueError, 'one narrator voice'):
            m.assemble(self.fx.args(spans, 'out-native-mixed'))
        # The canonical base accepted this mix of a native and a legacy Oliver unit; the guard is new.
        legacy, _ = self.fx.receipt(OLIVER, cues=('01_1',), v2=False, name='legacy-01_1.json')
        spans = self.fx.spans([self.fx.record('01_1', legacy), self.fx.record('01_2', native)], name='base-mixed.json')
        base.assemble(self.fx.args(spans, 'out-base-mixed'))
        with self.assertRaisesRegex(ValueError, 'one narrator voice'):
            m.assemble(self.fx.args(spans, 'out-patched-mixed'))

    def test_voice_swapped_after_assembly_refused_by_verify(self):
        per_cue = {}
        for cue in SCRIPT:
            per_cue[cue], _ = self.fx.receipt(FREDERICK, cues=(cue,), name=f'{cue}-own.json')
        spans = self.fx.spans([self.fx.record(c, per_cue[c]) for c in SCRIPT], name='own.json')
        m.assemble(self.fx.args(spans, 'out-own'))
        out = self.fx.root / 'out-own'
        self.assertTrue(m.verify(out, self.fx.script, self.fx.units, chapter='01'))
        # A complete, internally valid legacy Oliver receipt replaces one Frederick receipt.
        _, oliver = self.fx.receipt(OLIVER, cues=('01_2',), v2=False, name='swap.json')
        per_cue['01_2'].write_text(json.dumps(oliver))
        m.unit_spans(self.fx.plan['units'][1], self.script, self.fx.record('01_2', per_cue['01_2']))
        with self.assertRaisesRegex(ValueError, 'one narrator voice'):
            m.verify(out, self.fx.script, self.fx.units, chapter='01')

    def test_incorrect_text(self):
        self.refuse('external source, text', body=self.row0(text='Here is a smooth curve. It bends twice.'))
        self.refuse('external source, text', body=self.row0(text=SCRIPT['01_1'] + ' '))
        request = self.fx.request['01_1']
        original = request.read_text()
        for change in ({'text': 'Here is a smooth curve. It bends twice.'}, {'cue': '01_2'}, {'text': None}):
            with self.subTest(change=change):
                self.fx.write_request('01_1', FREDERICK[1], change)
                self.refuse('cue, text and voice exactly', body=self.row0(request_sha256=m.cache.digest(request)))
        request.write_text(original)
        changed_unit = dict(self.unit, text='Here is a smooth curve. It bends twice.')
        self.refuse('external source, text', unit=changed_unit)

    def test_hash_sample_and_rate_changes(self):
        h = self.body['records'][0]
        self.refuse('external source, text', body=self.row0(wav_sha256='0' * 64))
        self.refuse('external source, text', record=dict(self.rec, sha256='0' * 64))
        self.refuse('external source, text', body=self.row0(source_sha256='0' * 64))
        self.refuse('external source, text', body=self.row0(samples=h['samples'] - 1))
        self.refuse('external source, text', body=self.row0(rate=48000))
        self.refuse('external request file changed', body=self.row0(request_sha256='0' * 64))
        copy_wav = self.fx.root / 'identical-copy.wav'
        shutil.copyfile(self.fx.wav['01_1'], copy_wav)
        self.refuse('external source, text', body=self.row0(wav=str(copy_wav)))
        download = self.fx.download['01_1']
        saved = download.read_bytes()
        download.write_bytes(saved[:-2] + b'\x01\x00')
        self.refuse('external source, text')
        download.write_bytes(saved)
        request = self.fx.request['01_1']
        saved = request.read_text()
        request.write_text(saved.replace('Eleven v4', 'Eleven v3'))
        self.refuse('external request file changed')
        request.write_text(saved)
        decoded = self.fx.wav['01_1']
        saved = decoded.read_bytes()
        decoded.write_bytes(saved[:-2] + b'\x02\x00')
        self.refuse('external source, text')
        decoded.write_bytes(saved)
        m.unit_spans(self.unit, self.script, self.rec)  # every restore leaves the baseline valid

    def test_cue_scope_unchanged(self):
        self.refuse('one complete cue without context crop', record=dict(self.rec, context_after='01_2'))
        two = {'id': '01u1', 'line_ids': ['01_1', '01_2'], 'text': SCRIPT['01_1'] + ' ' + SCRIPT['01_2']}
        self.refuse('one complete cue without context crop', unit=two)


class DeliveryTests(Base):
    """narration_delivery's external-plan gate, directly and through its real CLI."""

    def plan_root(self, receipts):
        root = self.fx.root / 'plan'
        narration = root / 'narration'
        narration.mkdir(parents=True)
        canonical = root / 'canonical.json'
        canonical.write_text(json.dumps({'direction_contract': {'topic': 'example'}}))
        (narration / 'external-units.json').write_text(json.dumps(
            {'direction_contract': {'topic': 'example'}, 'units': [{'line_ids': [c]} for c in SCRIPT]}))
        (narration / 'assembly.json').write_text(json.dumps({'manifest': {'units': [
            {'generation_receipt': str(r)} for r in receipts]}}))
        return root, narration, canonical

    def test_external_plan_accepts_exact_labels_only(self):
        fred, body = self.fx.receipt(FREDERICK)
        root, narration, canonical = self.plan_root([fred])
        self.assertEqual(delivery.external_plan(root, narration, canonical), narration / 'external-units.json')
        oliver, _ = self.fx.receipt(OLIVER, v2=False, name='legacy.json')
        (narration / 'assembly.json').write_text(json.dumps({'manifest': {'units': [{'generation_receipt': str(oliver)}]}}))
        self.assertEqual(delivery.external_plan(root, narration, canonical), narration / 'external-units.json')
        (narration / 'assembly.json').write_text(json.dumps({'manifest': {'units': [{'generation_receipt': str(fred)}]}}))
        for change, pattern in (({'schema': None}, 'unrecognised external UI'), ({'source': 'other', 'schema': None}, 'UI source'),
                                ({'voice': 'Frederick Surrey'}, 'provider or voice')):
            with self.subTest(change=change):
                bad = {k: v for k, v in dict(body, **change).items() if v is not None}
                fred.write_text(json.dumps(bad))
                with self.assertRaisesRegex(ValueError, pattern):
                    delivery.external_plan(root, narration, canonical)

    def cli_repo(self, label_voice, v2):
        """A miniature repository around the patched tools, as tools/test_narration_delivery.py builds one."""
        repo = self.fx.root / 'repo'
        tools = repo / 'tools'
        tools.mkdir(parents=True)
        for name in ('narration_delivery.py', 'coherent_narration.py', 'narration_cache.py'):
            shutil.copy2(PATCHED / name, tools / name)
        topic = repo / 'explainer_corpus/example'
        topic.mkdir(parents=True)
        shutil.copy2(self.fx.script, topic / 'script.tsv')
        directions = topic / 'directions.json'
        directions.write_text(json.dumps({'overrides': [{'video': 'example', 'cue': '01_1', 'after': 'Here is a smooth curve'}]}))
        self.fx.plan['direction_contract'] = {'schema_version': 1, 'topic': 'example',
            'script_sha256': m.cache.digest(self.fx.script),
            'map': {'path': 'explainer_corpus/example/directions.json', 'sha256': m.cache.digest(directions)}}
        self.fx.units.write_text(json.dumps(self.fx.plan))
        shutil.copy2(self.fx.units, topic / 'narration_units.json')
        receipts = {c: self.fx.receipt(label_voice, cues=(c,), v2=v2, name=f'cli-{c}.json')[0] for c in SCRIPT}
        evidence = self.fx.root / 'review.txt'
        evidence.write_text('SYNTHETIC retained-boundary fixture; no listening.')
        spans = self.fx.root / 'cli-spans.json'
        spans.write_text(json.dumps({'schema_version': 1, 'units_sha256': m.cache.digest(self.fx.units),
            'boundary_review': {'reviewer': 'synthetic-test', 'basis': 'synthetic', 'listening': 'unreviewed'},
            'units': [dict(self.fx.record(c, receipts[c]), gap_after_samples=[0] * len(ENDS[c])) for c in SCRIPT],
            'direction_dispositions': [{'override_index': 0, 'kind': 'retained_source', 'reason': 'Keep synthetic source.',
                                        'evidence': {'path': str(evidence), 'sha256': m.cache.digest(evidence)}}]}))
        old = m.REPO
        m.REPO = repo
        try:
            out = self.fx.root / 'narration'
            m.assemble(types.SimpleNamespace(script=topic / 'script.tsv', units=self.fx.units, spans=spans,
                                             out=out, chapter='01', require_directions=True))
        finally:
            m.REPO = old
        shutil.copy2(self.fx.units, out / 'external-units.json')
        return tools, topic, out, receipts

    def run_cli(self, tools, topic, out):
        cmd = [sys.executable, '-I', '-B', str(tools / 'narration_delivery.py'), '--topic', str(topic),
               '--narration', str(out), '--consumer', 'render', '--gain-db', '0', '--chapter-key', '01_first']
        return subprocess.run(cmd, cwd='/', text=True, capture_output=True, timeout=60)

    def test_frederick_render_consumer_accepts_external_plan(self):
        tools, topic, out, _ = self.cli_repo(FREDERICK, True)
        result = self.run_cli(tools, topic, out)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn('"plan_mode": "external-ui"', result.stdout)
        self.assertIn('"listening_acceptance": false', result.stdout)

    def test_render_consumer_refuses_voice_swapped_receipt(self):
        tools, topic, out, receipts = self.cli_repo(OLIVER, False)
        result = self.run_cli(tools, topic, out)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        manifest = json.loads((out / 'assembly.json').read_text())['manifest']
        self.assertEqual(manifest['units'][1]['generation_receipt'], str(receipts['01_2']))
        # Replace the second unit's receipt with an otherwise valid Frederick one: same WAV, other voice.
        _, fred = self.fx.receipt(FREDERICK, cues=('01_2',), name='cli-swap.json')
        saved = receipts['01_2'].read_text()
        receipts['01_2'].write_text(json.dumps(fred))
        m.unit_spans(self.fx.plan['units'][1], m.read_script(self.fx.script),
                     self.fx.record('01_2', receipts['01_2']))  # the unit alone still passes
        result = self.run_cli(tools, topic, out)
        receipts['01_2'].write_text(saved)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('one narrator voice', result.stdout + result.stderr)
        self.assertEqual(self.run_cli(tools, topic, out).returncode, 0)


if __name__ == '__main__':
    unittest.main(argv=[sys.argv[0], '-v'])
