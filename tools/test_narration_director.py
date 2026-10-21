"""Behavior tests for diagnostic integrity; synthetic signals are not speech QA."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import wave

import numpy as np

spec = importlib.util.spec_from_file_location("director", Path(__file__).with_name("narration_director.py"))
d = importlib.util.module_from_spec(spec)
spec.loader.exec_module(d)


class DirectorTests(unittest.TestCase):
    def test_transcription_mismatch_stays_unmatched(self):
        self.assertEqual(d.align("a large gap".split(), "a wrong gap".split()), {0: 0, 2: 2})

    def test_numeric_expansion_retains_coarse_timestamp(self):
        words = d.asr_words({"segments": [{"start_sec": 1, "end_sec": 2, "text": "1,000"}]})
        self.assertEqual([w["text"] for w in words], ["one", "thousand"])
        self.assertTrue(all(w["start"] == 1 and w["end"] == 2 and w["coarse"] for w in words))

    def test_invalid_asr_time_is_rejected(self):
        with self.assertRaises(ValueError):
            d.asr_words({"segments": [{"start_sec": 2, "end_sec": 1, "text": "wrong"}]})

    def test_protected_question_is_not_shortened(self):
        self.assertEqual(d.classify([4, 4.2], [.7, 1.7], protected=True), "protected_visual_hold")
        self.assertEqual(d.classify([4, 4.2], [.7, 1.7], uncertain=True), "alignment_review")

    def test_threshold_disagreement_is_not_definite_defect(self):
        self.assertEqual(d.classify([.1, .6], [.4, .9]), "within_proposal")

    def test_phrase_override_requires_exact_unambiguous_location(self):
        direction = {"policy": d.POLICY, "overrides": [{"video": "a", "cue": "01_1", "after": "one", "role": "step", "range_s": [.5, 1], "reason": "Compute first."}]}
        with self.assertRaises(ValueError):
            d.directions("a", {"01_1": "One plus one."}, direction)
        direction['overrides'][0]['word_index'] = 0
        resolved = d.directions('a', {'01_1': 'One plus one.'}, direction)
        self.assertEqual(resolved[('01_1', 0)]['basis'], 'authored')
        self.assertEqual(resolved[('01_1', 2)]['basis'], 'heuristic')

    def test_waveform_recovers_gap_smoothed_into_asr_words(self):
        rate = 24000
        tone = .2 * np.sin(2 * np.pi * 220 * np.arange(rate) / rate)
        signal = np.concatenate([tone, np.zeros(rate // 2), tone])
        runs = d.quiet_runs(signal, rate, 30)
        gap, location = d.boundary_quiet(runs, {"start": 0, "end": 1.25}, {"start": 1.25, "end": 2.5})
        self.assertAlmostEqual(gap, .5, delta=.03)
        self.assertAlmostEqual(location[0], 1, delta=.03)

    def fixture(self, folder):
        rate = 24000
        tone = (.15 * np.sin(2 * np.pi * 220 * np.arange(rate) / rate) * 32767).astype('<i2').tobytes()
        samples = b'\0\0' * 2400 + tone + b'\0\0' * 4800
        for name in ('source', 'cue1', 'cue2'):
            with wave.open(str(folder / (name + '.wav')), 'wb') as w:
                w.setparams((1, 2, rate, 0, 'NONE', 'not compressed'))
                w.writeframes(samples)
        script = folder / 'script.tsv'
        script.write_text('01_1\tOne two.\n01_2\tOne two.\n')
        source = str(folder / 'source.wav')
        part = {"sentence": "One two.", "source": source, "source_sha256": d.digest(source), "source_in_sample": 0, "source_out_sample": 31200, "out_samples": 31200, "gap_after_samples": 0}
        report = {"meta": {"rate": rate, "script_sha256": d.digest(script)}, "lines": {"01_1": [part], "01_2": [part]}}
        # Distinct takes avoid mapping identical words twice onto one recording.
        source2 = folder / 'source2.wav'
        source2.write_bytes(Path(source).read_bytes())
        report['lines']['01_2'] = [dict(part, source=str(source2))]
        report_path = folder / 'report.json'
        report_path.write_text(json.dumps(report))
        timeline = folder / 'timeline'
        timeline.mkdir()
        (timeline / '01.timeline.tsv').write_text(f'chapter\t01\nfps\t60\nframes\t240\nline\t01_1\t0\t0\t{folder / "cue1.wav"}\nline\t01_2\t120\t0\t{folder / "cue2.wav"}\n')
        asr = folder / 'asr.ndjson'
        result = {'transcript': 'One two.', 'segments': [{'start_sec': .1, 'end_sec': .6, 'text': 'One'}, {'start_sec': .6, 'end_sec': 1.1, 'text': 'two.'}]}
        asr.write_text('\n'.join(json.dumps({'input': s, 'status': 'ok', 'report': {'result': result}}) for s in (source, str(source2))))
        master = folder / 'master.mp4'
        master.write_bytes(b'opaque fixture for identity checking, not a video decode')
        video = dict(id='a', script=str(script), report=str(report_path), timeline=str(timeline), master=str(master), master_sha256=d.digest(master), asr=[str(asr)])
        return video, d.draft_map({'videos': [video]})

    def test_scene_gap_counts_edge_quiet_once(self):
        with tempfile.TemporaryDirectory() as tmp:
            v, direction = self.fixture(Path(tmp))
            audit = d.audit_video(v, direction)
            boundary = next(b for b in audit['boundaries'] if b['cue'] != b['next_cue'])
            self.assertAlmostEqual(boundary['scene_added_s'], .7)
            self.assertAlmostEqual(boundary['observed_s'][0], 1.0, delta=.04)
            self.assertAlmostEqual(boundary['counterfactual_without_scene_s'][0], .3, delta=.04)

    def test_same_duration_pcm_corruption_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp)
            v, direction = self.fixture(folder)
            cue = folder / 'cue1.wav'
            content = bytearray(cue.read_bytes())
            content[-10] ^= 1
            cue.write_bytes(content)
            with self.assertRaisesRegex(ValueError, 'PCM does not reconstruct'):
                d.audit_video(v, direction)

    def test_changed_script_cannot_reuse_authored_map(self):
        with tempfile.TemporaryDirectory() as tmp:
            v, direction = self.fixture(Path(tmp))
            Path(v['script']).write_text('01_1\tA different thought.\n')
            with self.assertRaisesRegex(ValueError, 'different script'):
                d.validate_map({'videos': [v]}, direction)

    def test_overlapping_cues_not_hidden_by_quiet_tails(self):
        with tempfile.TemporaryDirectory() as tmp:
            v, direction = self.fixture(Path(tmp))
            p = Path(v['timeline'])/'01.timeline.tsv'
            p.write_text(p.read_text().replace('01_2\t120', '01_2\t72'))
            result = d.audit_video(v, direction)
            boundary = next(b for b in result['boundaries'] if b['cue'] != b['next_cue'])
            self.assertGreater(boundary['observed_s'][0], 0)
            self.assertEqual(boundary['kind'], 'overlapping_cues')

    def test_missing_asr_is_explicit_not_passed(self):
        with tempfile.TemporaryDirectory() as tmp:
            v, direction = self.fixture(Path(tmp))
            v['asr'] = []
            result = d.audit_video(v, direction)
            self.assertEqual(result['coverage']['missing_asr_sources'], 2)
            self.assertEqual(result['coverage']['usable_timed_words'], 0)
            self.assertTrue(all(b['kind'] == 'alignment_review' for b in result['boundaries']))

    def test_master_crosscheck_rejects_changed_transcription(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp)
            raw = folder/'raw.ndjson'
            raw.write_text('{}\n')
            receipt = folder/'receipt.json'
            receipt.write_text(json.dumps({'exit': 0, 'raw_sha256': 'not-current'}))
            with self.assertRaisesRegex(ValueError, 'verification failed or changed'):
                d.verify_master_excerpts({'verification': {'receipt': str(receipt), 'raw': str(raw)}}, [])


if __name__ == '__main__':
    unittest.main()
