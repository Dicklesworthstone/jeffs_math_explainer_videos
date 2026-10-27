"""Actual CLI consumer checks on retained synthetic PCM, never listening tests."""
import importlib.util
from collections import Counter
import json
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import unittest
import narration_delivery

TOOLS = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('fixtures', TOOLS/'test_coherent_narration.py')
fixtures = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fixtures)


class DirectedSegmentTests(unittest.TestCase):
    def test_prior_sentence_gaps_multiple_insertions_and_display_tokens(self):
        report = {'meta': {'rate': 100, 'direction_contract': {'binding': {'resolved': [
            {'override_index': 0, 'end_word_index': 2},
            {'override_index': 1, 'end_word_index': 5}]}}}, 'lines': {'x': [
            {'sentence': 'Warm up.', 'out_samples': 100, 'gap_after_samples': 50},
            {'sentence': 'One twenty-one calls remain.', 'source_in_sample': 200,
             'out_samples': 850, 'gap_after_samples': 0, 'intra_sentence_pauses': [
                 {'override_index': 0, 'source_sample': 400, 'inserted_samples': 20},
                 {'override_index': 1, 'source_sample': 800, 'inserted_samples': 30}]}]}}
        # Independent values from the retained native caption regression;
        # add a 12.5-second whole-film/cue placement.
        got = narration_delivery.directed_segments(report, 'x', 12.5)
        self.assertEqual(got, [
            {'cue': 'x', 'start_ms': 16200, 'end_ms': 20500,
             'text': 'twenty-one calls', 'preceding_text': 'One'},
            {'cue': 'x', 'start_ms': 20500, 'end_ms': 22500,
             'text': 'remain.', 'preceding_text': 'twenty-one calls'}])
        held = narration_delivery.directed_segments(report, 'x', 12.5, held_end=24.0)
        self.assertEqual(held[:-1], got[:-1])
        self.assertEqual(held[-1], dict(got[-1], end_ms=24000))
        # The tokenizer splits twenty-one, but its displayed word must not.
        report['meta']['direction_contract']['binding']['resolved'][1]['end_word_index'] = 3
        with self.assertRaisesRegex(ValueError, 'splits a display word'):
            narration_delivery.directed_segments(report, 'x', 12.5)
        report['meta']['direction_contract']['binding']['resolved'][1]['end_word_index'] = 5
        report['lines']['x'][1]['sentence'] += ' Several more words.' * 12
        self.assertEqual(narration_delivery.directed_segments(report, 'x', 12.5), got[:1])


class SegmentedScoreTests(unittest.TestCase):
    def test_exact_parts_and_every_bound_placement_are_required(self):
        import hashlib
        import tempfile
        root = Path(tempfile.mkdtemp(prefix='score-parts-contract-'))
        score = root/'stems'
        score.mkdir()
        manifest = score/'01_first.music.f32.parts.tsv'
        lines = ['score-parts\t1']
        log = root/'cue.tsv'
        logs = []
        zero = '0x'+struct.pack('>d',0.0).hex()
        for i in range(2):
            path = score/f'piece{i}.wav'
            # Two small, distinct IEEE-float stereo cues, at consecutive samples.
            data = struct.pack('<4f',0.1*i,0.2,0.3,0.4)
            body = b'WAVEfmt '+struct.pack('<IHHIIHH',16,3,2,48000,384000,8,32)+b'data'+struct.pack('<I',len(data))+data
            path.write_bytes(b'RIFF'+struct.pack('<I',len(body))+body)
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
            lines.append(f'{i*2}\t2\t48000\t{digest}\t{path.name}')
            offset = i*2/48000
            bits = '0x'+struct.pack('>d',offset).hex()
            logs.append(f'{path}\t{digest}\t{path.stat().st_size}\t0\t60\t{offset}\t{bits}\t0\t{zero}\t48000\t2\t2\t{i*2}')
        original = '\n'.join(lines)+'\n'
        manifest.write_text(original)
        log.write_text('\n'.join(logs)+'\n')
        narration_delivery.cue_log(log,60,Counter(),[],score,'01_first')
        for wrong in [logs[:1], logs+[logs[0]], [logs[0], logs[1].replace('\t2\n','\t3\n')+'\tbad']]:
            log.write_text('\n'.join(wrong)+'\n')
            with self.assertRaises(ValueError):
                narration_delivery.cue_log(log,60,Counter(),[],score,'01_first')
        log.write_text('\n'.join(logs)+'\n')
        for changed in [original.replace('2\t2\t48000','3\t2\t48000'), original.replace('piece0.wav','../piece0.wav'), original.replace('48000','24000')]:
            manifest.write_text(changed)
            with self.assertRaises(ValueError):
                narration_delivery.cue_log(log,60,Counter(),[],score,'01_first')
        manifest.write_text(original)
        (score/'piece1.wav').write_bytes(b'changed')
        with self.assertRaises(ValueError):
            narration_delivery.cue_log(log,60,Counter(),[],score,'01_first')


class DeliveryTests(unittest.TestCase):
    def setUp(self):
        self.fixture = fixtures.CoherentTests()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.args = self.fixture.bound_assembly()
        self.root = self.fixture.root.resolve()
        self.topic = self.root/'explainer_corpus/example'
        shutil.copy2(self.fixture.script, self.topic/'script.tsv')
        shutil.copy2(self.fixture.units, self.topic/'narration_units.json')
        tools = self.root/'tools'
        tools.mkdir()
        for name in ('narration_delivery.py', 'coherent_narration.py', 'narration_cache.py'):
            shutil.copy2(TOOLS/name, tools/name)
        self.helper = tools/'narration_delivery.py'
        self.timelines = self.root/'timelines'
        self.timelines.mkdir()
        outputs = json.loads((self.args.out/'assembly.json').read_text())['outputs']
        timeline = ['chapter\t01_first', 'fps\t15', 'frames\t90']
        logs = []
        zero = '0x'+struct.pack('>d', 0.0).hex()
        for i, (cue, info) in enumerate(outputs.items()):
            wav = self.args.out/f'{cue}.wav'
            frame = i*15
            timeline.append(f'line\t{cue}\t{frame}\t0\t{wav}')
            logs.append('\t'.join(map(str, (wav, info['sha256'], info['bytes'], frame, 15,
                0, zero, 0, zero, info['sample_rate'], info['channels'], info['frames'], frame*3200))))
        self.timeline = self.timelines/'01_first.timeline.tsv'
        self.timeline.write_text('\n'.join(timeline)+'\n')
        self.log = self.timelines/'01_first.cuelog.tsv'
        self.log.write_text('\n'.join(logs)+'\n')

    def run_cli(self, consumer, success=True, extra=()):
        cmd = [sys.executable, '-I', '-B', str(self.helper), '--topic', str(self.topic),
               '--narration', str(self.args.out), '--consumer', consumer, '--gain-db', '0',
               '--chapter-key', '01_first', '--timelines', str(self.timelines), *extra]
        result = subprocess.run(cmd, cwd='/', text=True, capture_output=True, timeout=30)
        self.assertEqual(result.returncode == 0, success, result.stdout+result.stderr)
        return result

    def test_all_four_actual_consumers_accept_bound_synthetic_input(self):
        for consumer in ('render', 'captions', 'score', 'master'):
            with self.subTest(consumer=consumer):
                result = self.run_cli(consumer)
                self.assertIn('"listening_acceptance": false', result.stdout)

    def test_external_script_and_units_cannot_be_stale(self):
        script = self.topic/'script.tsv'
        original = script.read_bytes()
        script.write_bytes(original.replace(b'Four pulses', b'Five pulses'))
        self.run_cli('render', False)
        script.write_bytes(original)
        units = self.topic/'narration_units.json'
        plan = json.loads(units.read_text())
        plan['units'][0]['text'] = 'Invented replacement.'
        units.write_text(json.dumps(plan))
        self.run_cli('render', False)

    def test_self_rehashed_missing_direction_refused_by_consumer(self):
        report = self.args.out/'report.json'
        data = json.loads(report.read_text())
        data['meta']['direction_contract']['dispositions'] = []
        report.write_text(json.dumps(data))
        receipt = self.args.out/'assembly.json'
        data = json.loads(receipt.read_text())
        data['report_sha256'] = fixtures.m.cache.digest(report)
        receipt.write_text(json.dumps(data))
        self.run_cli('render', False)

    def test_timeline_substitution_omission_gain_and_duplicate_refused(self):
        original = self.timeline.read_text()
        lines = original.splitlines()
        for mutation in (original.replace('01_1.wav', '01_2.wav'),
                         '\n'.join(lines[:-1])+'\n', original+'\n'+lines[-1]+'\n',
                         original.replace('\t0\t/', '\t1\t/')):
            self.timeline.write_text(mutation)
            for consumer in ('captions', 'score', 'master'):
                with self.subTest(consumer=consumer, mutation=mutation):
                    self.run_cli(consumer, False)
        self.timeline.write_text(original)

    def test_cue_log_actual_path_timing_gain_and_coverage_are_bound(self):
        original = self.log.read_text()
        rows = [line.split('\t') for line in original.splitlines()]
        for column, value in ((0, rows[1][0]), (6, '0x3ff0000000000000'),
                              (8, '0x3ff0000000000000'), (12, '1')):
            changed = [row.copy() for row in rows]
            changed[0][column] = value
            self.log.write_text('\n'.join('\t'.join(row) for row in changed)+'\n')
            self.run_cli('master', False)
        for text in ('\t'.join(rows[0])+'\n', original+'\t'.join(rows[0])+'\n'):
            self.log.write_text(text)
            self.run_cli('master', False)
        self.log.write_text(original)

    def test_unknown_or_duplicate_chapter_refused(self):
        self.run_cli('render', False, ('--chapter-key', '01_first'))
        self.run_cli('score', False, ('--chapter-key', '02_absent'))

    def test_explicit_partial_chapter_cannot_score_or_master_whole_film(self):
        fixture = self.fixture
        fixture.script.write_text(fixture.script.read_text()+'02_1\tAnother chapter.\n')
        digest = fixtures.m.cache.digest(fixture.script)
        fixture.plan['script_sha256'] = digest
        fixture.plan['direction_contract']['script_sha256'] = digest
        fixture.plan['units'].append({'id': '02u1', 'line_ids': ['02_1'], 'text': 'Another chapter.'})
        fixture.save_plan()
        manifest = json.loads(self.args.spans.read_text())
        manifest['units_sha256'] = fixtures.m.cache.digest(fixture.units)
        spans = self.root/'partial-spans.json'
        spans.write_text(json.dumps(manifest))
        self.args.spans = spans
        self.args.out = self.root/'partial-out'
        fixtures.m.assemble(self.args)
        shutil.copy2(fixture.script, self.topic/'script.tsv')
        shutil.copy2(fixture.units, self.topic/'narration_units.json')
        self.run_cli('render')
        self.run_cli('captions')  # retained timeline WAVs are byte-identical
        self.run_cli('score', False)
        self.run_cli('master', False)

    def test_extra_audio_requires_explicit_score_or_listen_declaration(self):
        score = self.root/'stems'
        score.mkdir()
        stem = score/'01_first.music.f32.wav'
        # Separate synthetic source audio, distinct from both verified cue WAVs.
        manifest = json.loads((self.args.out/'assembly.json').read_text())['manifest']
        source = Path(manifest['units'][0]['wav'])
        shutil.copy2(source, stem)
        row = self.log.read_text().splitlines()[0].split('\t')
        row[0], row[1] = str(stem), fixtures.m.cache.digest(stem)
        self.log.write_text(self.log.read_text()+'\t'.join(row)+'\n')
        self.run_cli('master', False)
        self.run_cli('master', extra=('--score', str(score)))
        self.timeline.write_text(self.timeline.read_text()+f'listen\t0\t1\tsynthetic\t{stem}\n')
        self.run_cli('master')

    def lab(self):
        lab = self.root/'narration_lab/review'
        lab.mkdir(parents=True)
        plan = lab/'units.json'
        shutil.copy2(self.topic/'narration_units.json', plan)
        self.timelines = lab/'preview'
        self.timelines.mkdir()
        shutil.copy2(self.timeline, self.timelines/self.timeline.name)
        return plan, ['--review-units', str(plan), '--output', str(self.timelines),
                      '--width', '1920', '--height', '1080', '--fps', '15']

    def test_explicit_lab_plan_works_without_canonical_promotion(self):
        plan, flags = self.lab()
        canonical = self.topic/'narration_units.json'
        data = json.loads(canonical.read_text())
        data['units'][0]['text'] = 'Different canonical plan.'
        canonical.write_text(json.dumps(data))
        for consumer in ('render', 'captions'):
            self.run_cli(consumer, False)
            result = self.run_cli(consumer, extra=flags)
            self.assertIn('"plan_mode": "lab-review"', result.stdout)
            self.assertIn('"promotion": false', result.stdout)
        self.assertEqual(json.loads(canonical.read_text()), data)
        script = self.topic/'script.tsv'
        script.write_text(script.read_text().replace('Four pulses', 'Five pulses'))
        self.run_cli('render', False, flags)

    def test_lab_rejects_final_consumers_scope_and_resource_expansion(self):
        _, flags = self.lab()
        for consumer in ('score', 'master'):
            self.run_cli(consumer, False, flags)
        for extra in (['--chapter-key', '02_second'], ['--width', '3840'],
                      ['--height', '2160'], ['--fps', '60'], ['--fps', '0'],
                      ['--width', '0'], ['--score', str(self.root/'stems')]):
            self.run_cli('render', False, flags+extra)
        path = self.timelines/self.timeline.name
        path.write_text(path.read_text().replace('fps\t15', 'fps\t60'))
        self.run_cli('captions', False, flags)

    def test_lab_requires_lexical_and_resolved_containment(self):
        plan, flags = self.lab()
        outside = self.root/'outside'
        outside.mkdir()
        shutil.copy2(plan, outside/'units.json')
        link = plan.parent/'escape'
        link.symlink_to(outside, target_is_directory=True)
        for target, option in (('units.json', '--review-units'), ('output', '--output')):
            for path in (outside/target, link/target,
                         plan.parent/'..'/'review'/target, self.root/'narration_lab'):
                self.run_cli('render', False, flags+[option, str(path)])
        self.run_cli('captions', False, flags+['--timelines', str(outside)])
        # Missing explicit output is not a default production-directory escape.
        self.run_cli('render', False, ['--review-units', str(plan)])


if __name__ == '__main__':
    unittest.main()
