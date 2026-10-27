"""Standalone release checks on synthetic PCM; never final-film or hearing proof."""
import copy
import json
from pathlib import Path
import shutil
import struct
import subprocess
import sys
import unittest
import wave

sys.dont_write_bytecode = True
TOOLS = Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS))
import release
import final_check
import test_coherent_narration as fixtures
import test_release as cuts


class ReleaseContract(unittest.TestCase):
    directed = False
    prediction = False

    def setUp(self):
        class Fixture(fixtures.CoherentTests):
            def bind_direction(inner):
                inner.direction = inner.root/'explainer_corpus/inspection_paradox/docs/directions.json'
                inner.direction.parent.mkdir(parents=True)
                data = {'overrides': [
                    {'video': 'inspection_paradox', 'cue': '01_1', 'after': 'Four pulses'}]}
                if self.prediction:
                    data['visual_holds'] = [{'video': 'inspection_paradox', 'cue': '01_1',
                        'next_cue': '01_2', 'kind': 'prediction',
                        'reason': 'Synthetic authored prediction; no listening claim.'}]
                inner.direction.write_text(json.dumps(data))
                inner.plan['direction_contract'] = {'schema_version': 1, 'topic': 'inspection_paradox',
                    'script_sha256': fixtures.m.cache.digest(inner.script),
                    'map': {'path': 'explainer_corpus/inspection_paradox/docs/directions.json',
                            'sha256': fixtures.m.cache.digest(inner.direction)}}
                inner.save_plan()

        self.fixture = Fixture()
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        if self.directed:
            self.args, manifest, _ = self.fixture.intra_fixture()
            directions = json.loads(self.fixture.direction.read_text())
            for item in directions['overrides']:
                item['video'] = 'inspection_paradox'
            self.fixture.direction.write_text(json.dumps(directions))
            self.fixture.plan['direction_contract']['map']['sha256'] = fixtures.m.cache.digest(self.fixture.direction)
            self.fixture.save_plan()
            manifest['units_sha256'] = fixtures.m.cache.digest(self.fixture.units)
            self.args.spans.write_text(json.dumps(manifest))
            fixtures.m.assemble(self.args)
        else:
            self.args = self.fixture.bound_assembly()
        self.root = self.fixture.root.resolve()
        self.topic = self.root/'explainer_corpus/inspection_paradox'
        script = self.fixture.script.read_text()
        self.fixture.script.write_text(''.join(script.replace('01_', f'{i:02}_') for i in range(1, 4)))
        unit = self.fixture.plan['units'][0]
        self.fixture.plan['units'] = [dict(unit, id=f'{i:02}u1',
            line_ids=[cue.replace('01_', f'{i:02}_') for cue in unit['line_ids']]) for i in range(1, 4)]
        digest = fixtures.m.cache.digest(self.fixture.script)
        self.fixture.plan['script_sha256'] = digest
        self.fixture.plan['direction_contract']['script_sha256'] = digest
        self.fixture.save_plan()
        manifest = json.loads(self.args.spans.read_text())
        record = manifest['units'][0]
        manifest['units'] = [dict(record, id=f'{i:02}u1') for i in range(1, 4)]
        for row in manifest['units'][1:]:
            row.pop('intra_sentence_pauses', None)
        manifest['units_sha256'] = fixtures.m.cache.digest(self.fixture.units)
        self.args.spans = self.root/'full-spans.json'
        self.args.spans.write_text(json.dumps(manifest))
        self.args.out = self.root/'full-out'
        self.args.chapter = None
        fixtures.m.assemble(self.args)
        cuts.make_cut(self.root)
        shutil.copytree(self.root/'explainer_corpus/alpha', self.topic, dirs_exist_ok=True)
        shutil.copy2(self.fixture.script, self.topic/'script.tsv')
        shutil.copy2(self.fixture.units, self.topic/'narration_units.json')
        (self.topic/'docs/protected.tsv').write_text('Four pulses\tcaptions\tsynthetic fixture\n')
        (self.topic/'src').mkdir()
        shutil.copy2(TOOLS.parent/'explainer_corpus/inspection_paradox/src/main.rs', self.topic/'src/main.rs')
        gain_source = self.root/'crates/explainer_kit/src/narration.rs'
        gain_source.parent.mkdir(parents=True)
        shutil.copy2(TOOLS.parent/'crates/explainer_kit/src/narration.rs', gain_source)
        tools = self.root/'tools'
        tools.mkdir()
        for name in ('release.py', 'publish.py', 'final_check.py', 'youtube_package.py', 'description_numbers.py',
                     'narration_delivery.py', 'coherent_narration.py', 'narration_cache.py'):
            shutil.copy2(TOOLS/name, tools/name)
        self.final = self.topic/'renders/final'
        outputs = json.loads((self.args.out/'assembly.json').read_text())['outputs']
        zero = '0x'+struct.pack('>d', 0.0).hex()
        for i, (key, frames) in enumerate(cuts.TIMELINES, 1):
            lines = [f'chapter\t{key}', 'fps\t60', f'frames\t{frames}']
            logs = []
            for j, cue in enumerate((f'{i:02}_1', f'{i:02}_2')):
                info, frame = outputs[cue], j*60
                wav = self.args.out/f'{cue}.wav'
                lines.append(f'line\t{cue}\t{frame}\t0\t{wav}')
                logs.append('\t'.join(map(str, (wav, info['sha256'], info['bytes'], frame, 60,
                    0, zero, 0, zero, info['sample_rate'], info['channels'], info['frames'], frame*800))))
            (self.final/f'{key}.timeline.tsv').write_text('\n'.join(lines)+'\n')
            (self.final/f'{key}.cuelog.tsv').write_text('\n'.join(logs)+'\n')
        text = unit['text']
        srt = (f'1\n00:00:00,000 --> 00:00:05,000\n{text}\n\n'
               f'2\n00:01:01,500 --> 00:01:06,500\n{text}\n\n'
               f'3\n00:01:48,750 --> 00:01:53,750\n{text}\n')
        (self.final/'alpha.en.srt').write_text(srt)
        (self.final/'alpha.en.vtt').write_text(cuts.vtt(srt))

    def check(self):
        return release.check('inspection_paradox', self.final, root=self.root,
                             narration=self.args.out, probe=lambda _: dict(cuts.GOOD_PROBE))

    def cli(self, tool, *extra):
        args = ['check', 'inspection_paradox', '--finals', str(self.final)] if tool == 'release.py' else [
            'inspection_paradox', '--master', str(self.final/'alpha_master.mp4'), '--out', str(self.root/'bundle')]
        if tool == 'final_check.py':
            args = ['inspection_paradox', str(self.root), '--strict', '--no-loudness']
        return subprocess.run([sys.executable, '-I', '-B', str(self.root/'tools'/tool), *args,
            '--narration', str(self.args.out), *extra], cwd='/', text=True, capture_output=True, timeout=30)

    def refused(self, needle):
        bad, sheet = self.check()
        self.assertTrue(any(needle in message for message in bad), bad)
        self.assertEqual(sheet, [])

    def test_bound_synthetic_release_inputs_pass(self):
        # Only the media metadata probe is synthetic; coherent sample reconstruction
        # and the canonical master-input CLI run against actual fixture files.
        bad, sheet = self.check()
        self.assertEqual(bad, [])
        self.assertTrue(sheet)

    def test_script_plan_and_self_rehashed_direction_changes_refused(self):
        script = self.topic/'script.tsv'
        original = script.read_bytes()
        script.write_bytes(original.replace(b'negative', b'positive'))
        self.refused('canonical narration check failed')
        script.write_bytes(original)
        plan = self.topic/'narration_units.json'
        old_plan = plan.read_bytes()
        data = json.loads(old_plan)
        data['units'][0]['text'] = 'Invented replacement.'
        plan.write_text(json.dumps(data))
        self.refused('canonical narration check failed')
        plan.write_bytes(old_plan)
        report = self.args.out/'report.json'
        data = json.loads(report.read_text())
        data['meta']['direction_contract']['dispositions'] = []
        report.write_text(json.dumps(data))
        receipt = self.args.out/'assembly.json'
        data = json.loads(receipt.read_text())
        data['report_sha256'] = fixtures.m.cache.digest(report)
        receipt.write_text(json.dumps(data))
        self.refused('canonical narration check failed')

    def test_actual_cli_refuses_before_release_sheet_or_bundle(self):
        (self.topic/'script.tsv').write_text('01_1\tA different script.\n')
        for tool in ('release.py', 'publish.py', 'final_check.py'):
            with self.subTest(tool=tool):
                result = self.cli(tool)
                self.assertNotEqual(result.returncode, 0, result.stdout+result.stderr)
                self.assertIn('narration contract', result.stdout+result.stderr)
                self.assertNotIn('PASS ', result.stdout)
                self.assertFalse((self.root/'bundle').exists())

    def test_timeline_and_actual_cue_audio_are_bound(self):
        tl = self.final/'01_one.timeline.tsv'
        original = tl.read_text()
        tl.write_text(original.replace('01_1.wav', '01_2.wav'))
        self.refused('substitutes narration')
        tl.write_text(original)
        log = self.final/'01_one.cuelog.tsv'
        original = log.read_text()
        fields = original.splitlines()[0].split('\t')
        for index, value in ((0, str(self.args.out/'01_2.wav')), (12, '1')):
            changed = fields.copy()
            changed[index] = value
            log.write_text('\t'.join(changed)+'\n'+original.splitlines()[1]+'\n')
            self.refused('canonical narration check failed')
        log.rename(log.with_suffix('.tsv.before'))
        self.refused('01_one.cuelog.tsv')

    def test_all_five_topics_require_contract_even_when_plan_missing(self):
        for slug in release.CONTRACT_TOPICS:
            if slug == 'inspection_paradox':
                (self.topic/'narration_units.json').rename(self.topic/'units.before.json')
            else:
                path = self.root/'explainer_corpus'/slug/'src'
                path.mkdir(parents=True)
                shutil.copy2(self.topic/'src/main.rs', path/'main.rs')
            bad = release.narration_problems(slug, self.final, self.root, self.args.out)
            self.assertTrue(bad, slug)
            self.assertIn('canonical narration check failed', bad[0])

    def test_release_entry_points_do_not_accept_lab_plan_override(self):
        for tool in ('release.py', 'publish.py', 'final_check.py'):
            result = self.cli(tool, '--review-units', str(self.fixture.units))
            self.assertEqual(result.returncode, 2)
            self.assertIn('unrecognized arguments: --review-units', result.stderr)
            self.assertFalse((self.root/'bundle').exists())

    def test_final_fetch_carries_cue_logs_and_preserves_old_cut(self):
        clone = self.root/'remote'
        remote = clone/self.final.relative_to(self.root)
        shutil.copytree(self.final, remote)
        changed = remote/'01_one.cuelog.tsv'
        changed.write_text(changed.read_text()+'# retained test revision\n')
        old = (self.final/'01_one.cuelog.tsv').read_bytes()
        release.fetch('inspection_paradox', self.final, f'fixture:{clone}', cuts.LocalHost(), self.root)
        self.assertEqual((self.final/'01_one.cuelog.tsv').read_bytes(), changed.read_bytes())
        self.assertEqual((self.final.with_name('final_v1')/'01_one.cuelog.tsv').read_bytes(), old)

    def test_long_third_wave_chapter_has_no_default_cap_but_explicit_limit_applies(self):
        # Exercise the actual canonical input check, then inject only media
        # metadata. Other final diagnostics remain visible, not acceptance.
        for key, _ in cuts.TIMELINES:
            (self.final/f'{key}.mp4').write_bytes(b'synthetic probe placeholder')
        def probe(path, entries, stream=None):
            if stream == 'v:0':
                return {'streams': [dict(cuts.GOOD_PROBE, duration='100', nb_frames='6000')]}
            if stream == 'a:0':
                return {'streams': [{'duration': '100', 'sample_rate': '48000', 'channels': 2}]}
            return {'format': {'duration': '100', 'size': '1000000'}}
        for limit in (None, 85.0, 110.0):
            defects = final_check.check_final('inspection_paradox', self.root, loud=False,
                ch_max=limit, narration=self.args.out, probe_fn=probe)
            self.assertFalse(any('narration contract' in d for d in defects), defects)
            self.assertFalse(any("captions' source is missing" in d for d in defects), defects)
            self.assertTrue(any('longest' not in d and 'timeline' in d for d in defects), defects)
            self.assertEqual(any('is over' in d for d in defects), limit == 85.0)

    def test_invalid_explicit_limits_fail_before_inspection(self):
        for limit in (0, -1, float('nan'), float('inf'), -float('inf')):
            defects = final_check.check_final('inspection_paradox', self.root, ch_max=limit)
            self.assertEqual(defects, ['chapter limit must be finite and positive'])


class DirectedReleaseContract(ReleaseContract):
    directed = True

    def setUp(self):
        super().setUp()
        self.srt = ('1\n00:00:00,000 --> 00:00:00,012\nFour\n\n'
                    '2\n00:00:00,012 --> 00:00:00,027\npulses.\n\n'
                    '3\n00:00:00,028 --> 00:00:00,043\nThey\n\n'
                    '4\n00:00:00,043 --> 00:00:00,068\nreturn.\n\n'
                    '5\n00:00:01,000 --> 00:00:01,038\nDo not drop the negative.\n\n'
                    '6\n00:01:01,500 --> 00:01:06,500\nFour pulses. They return. Do not drop the negative.\n\n'
                    '7\n00:01:48,750 --> 00:01:53,750\nFour pulses. They return. Do not drop the negative.\n')
        self.set_captions(self.srt)

    def set_captions(self, srt):
        (self.final/'alpha.en.srt').write_text(srt)
        (self.final/'alpha.en.vtt').write_text(cuts.vtt(srt))

    def test_bound_short_directed_tails_pass_without_changing_pcm(self):
        before = {p.name: fixtures.m.cache.digest(p) for p in self.args.out.glob('*.wav')}
        bad, sheet = self.check()
        self.assertEqual(bad, [])
        self.assertTrue(sheet)
        self.assertEqual(before, {p.name: fixtures.m.cache.digest(p) for p in self.args.out.glob('*.wav')})

    def test_directed_caption_exception_requires_exact_text_and_endpoints(self):
        for old, new in (('00:00:00,012 --> 00:00:00,027', '00:00:00,013 --> 00:00:00,027'),
                         ('00:00:00,012 --> 00:00:00,027', '00:00:00,012 --> 00:00:00,026'),
                         ('pulses.', 'pulse.'), ('00:00:00,000 --> 00:00:00,012',
                                                '00:00:00,000 --> 00:00:00,011')):
            with self.subTest(mutation=new):
                self.set_captions(self.srt.replace(old, new, 1))
                bad, _ = self.check()
                self.assertTrue(any('tail of the sentence' in d for d in bad), bad)

    def test_directed_caption_exception_refuses_rehashed_unbound_claim(self):
        report = self.args.out/'report.json'
        data = json.loads(report.read_text())
        data['meta']['direction_contract']['dispositions'][0]['kind'] = 'retained_source'
        report.write_text(json.dumps(data))
        assembly = self.args.out/'assembly.json'
        receipt = json.loads(assembly.read_text())
        receipt['report_sha256'] = fixtures.m.cache.digest(report)
        assembly.write_text(json.dumps(receipt))
        self.refused('canonical narration check failed')


class FinalTiming(ReleaseContract):
    prediction = True

    def setUp(self):
        super().setUp()
        import test_final_check
        template = test_final_check.TestFinalCheck('test_fixture_pass')
        template.setUp()  # Retained fixture, no destructive tearDown.
        self.metadata_fixture = template.temp_dir
        self.probe_data = copy.deepcopy(template.probe_data)
        duration = sum(frames / 60 for _, frames in cuts.TIMELINES)
        for key in ('master_video', 'master_audio', 'master_format'):
            self.probe_data[key]['duration'] = str(duration)
        self.probe_data['master_video']['nb_frames'] = round(duration * 60)
        self.chapter_frames = dict(cuts.TIMELINES)
        for key, _ in cuts.TIMELINES:
            (self.final/f'{key}.mp4').write_bytes(b'synthetic metadata probe placeholder')
        self.timeline = self.final/'01_one.timeline.tsv'
        self.timeline.write_text(self.timeline.read_text()+'mark\tpredict\t30\n')
        parts = []
        def stamp(ms):
            seconds, ms = divmod(ms, 1000)
            minutes, seconds = divmod(seconds, 60)
            hours, minutes = divmod(minutes, 60)
            return f'{hours:02}:{minutes:02}:{seconds:02},{ms:03}'
        for chapter, offset in enumerate((0, 61500, 108750)):
            ends = ((0, 25, 'Four pulses.'),
                    (25, 1000 if chapter == 0 else 63, 'They return.'),
                    (1000, 1038, 'Do not drop the negative.'))
            for start, end, words in ends:
                parts.append(f'{len(parts)+1}\n{stamp(offset+start)} --> {stamp(offset+end)}\n{words}')
        self.srt = '\n\n'.join(parts)+'\n'
        self.set_captions(self.srt)

    def set_captions(self, srt):
        (self.final/'alpha.en.srt').write_text(srt)
        (self.final/'alpha.en.vtt').write_text(cuts.vtt(srt))

    def probe(self, path, entries, stream=None):
        p = Path(path)
        if p.suffix == '.wav':
            with wave.open(str(p), 'rb') as wav:
                return {'format': {'duration': str(wav.getnframes()/wav.getframerate())}}
        if '_master' in p.name:
            key = {'v:0': 'master_video', 'a:0': 'master_audio'}.get(stream, 'master_format')
            return {'streams': [self.probe_data[key]]} if stream else {'format': self.probe_data[key]}
        frames = self.chapter_frames[p.stem]
        if stream:
            return {'streams': [dict(self.probe_data['chapter_video'], nb_frames=frames)]}
        return {'format': {'duration': str(frames/60)}}

    def final_diagnostics(self, module=final_check):
        return module.check_final('inspection_paradox', self.root, loud=False,
                                  narration=self.args.out, probe_fn=self.probe)

    def test_prediction_hold_passes_without_altering_narration_or_captions(self):
        paths = list(self.args.out.glob('*.wav')) + [self.final/'alpha.en.srt', self.final/'alpha.en.vtt']
        before = {str(p): fixtures.m.cache.digest(p) for p in paths}
        self.assertEqual(self.final_diagnostics(), [])
        self.assertEqual(before, {str(p): fixtures.m.cache.digest(p) for p in paths})

    def test_prediction_hold_requires_correct_mark_window_and_exact_end(self):
        original = self.timeline.read_text()
        for mark in ('', 'mark\tpredict\t0\n', 'mark\tpredict\t60\n'):
            self.timeline.write_text(original.replace('mark\tpredict\t30\n', mark))
            defects = self.final_diagnostics()
            self.assertTrue(any('outside their audio' in d for d in defects), defects)
        self.timeline.write_text(original)
        for end in ('00:00:00,999', '00:00:01,001'):
            self.set_captions(self.srt.replace('00:00:00,025 --> 00:00:01,000',
                                               '00:00:00,025 --> '+end))
            defects = self.final_diagnostics()
            self.assertTrue(any('outside their audio' in d for d in defects), defects)
        self.set_captions(self.srt.replace('00:00:00,025 --> 00:00:01,000',
                                          '00:00:00,200 --> 00:00:01,000'))
        self.assertTrue(any('outside their audio' in d for d in self.final_diagnostics()))

    def test_vtt_endpoint_mutations_fail_final_and_release_checks(self):
        original = cuts.vtt(self.srt)
        for old, new in (('00:00:00.000', '00:00:00.001'), ('00:00:00.025', '00:00:00.024'),
                         ('00:00:00.025', 'not-a-time')):
            (self.final/'alpha.en.vtt').write_text(original.replace(old, new, 1))
            self.assertTrue(any('VTT' in d for d in self.final_diagnostics()))
            self.assertTrue(any('VTT' in d for d in self.check()[0]))


    def test_timeline_mark_without_bound_authored_hold_cannot_extend_caption(self):
        other = FinalTiming('test_prediction_hold_passes_without_altering_narration_or_captions')
        other.prediction = False
        other.setUp()
        self.addCleanup(other.doCleanups)
        defects = other.final_diagnostics()
        self.assertTrue(any('outside their audio' in d for d in defects), defects)


if __name__ == '__main__':
    unittest.main()
