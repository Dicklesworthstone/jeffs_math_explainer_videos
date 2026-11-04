"""Focused tests for caption split timing and refusal cases."""
import copy
import hashlib
import unittest
from unittest.mock import patch

import retime_captions_from_phrases as subject


class FakeAudio:
    def __enter__(self):
        return self

    def __exit__(self, *_):
        return False

    def getnframes(self):
        return 96000

    def getframerate(self):
        return 24000


class CaptionTimingTests(unittest.TestCase):
    def setUp(self):
        text = 'One two three four.'
        self.script = {'01_1': text}
        self.report = {'meta': {'rate': 24000}, 'lines': {'01_1': [
            {'sentence': text, 'out_samples': 96000, 'gap_after_samples': 0}]}}
        self.timing = {'cues': {'01_1': {
            'text_sha256': hashlib.sha256(text.encode()).hexdigest(),
            'wav_sha256': 'audio-hash', 'marks': [{'byte': 8, 'seconds': 1.6}]}}}
        self.placements = [('01_1', 1.0, 'audio.wav')]
        self.captions = [{'start': 1000, 'end': 3000, 'lines': ['One two']},
                         {'start': 3000, 'end': 6000, 'lines': ['three four.']}]

    def run_retime(self):
        with patch.object(subject, 'sha', return_value='audio-hash'), patch.object(subject.wave, 'open', return_value=FakeAudio()):
            return subject.retime(self.captions, self.script, self.report, self.timing, self.placements)

    def test_split_uses_mark_and_preserves_sentence_edges_and_hold(self):
        changed, skipped = self.run_retime()
        self.assertEqual((self.captions[0]['start'], self.captions[0]['end']), (1000, 2600))
        self.assertEqual((self.captions[1]['start'], self.captions[1]['end']), (2600, 6000))
        self.assertEqual(len(changed), 1)
        self.assertEqual(skipped, [])
        self.assertEqual(subject.parse_srt(subject.render(self.captions)), self.captions)
        self.assertTrue(subject.render(self.captions, True).startswith('WEBVTT\n\n'))

    def test_missing_mark_and_noncontiguous_split_stay_unchanged(self):
        original = copy.deepcopy(self.captions)
        self.timing['cues']['01_1']['marks'] = []
        changed, skipped = self.run_retime()
        self.assertEqual(self.captions, original)
        self.assertEqual((len(changed), len(skipped)), (0, 1))
        self.timing['cues']['01_1']['marks'] = [{'byte': 8, 'seconds': 1.6}]
        self.captions[1]['start'] = 3100
        changed, skipped = self.run_retime()
        self.assertEqual(self.captions[1]['start'], 3100)
        self.assertEqual((len(changed), len(skipped)), (0, 1))

    def test_changed_words_or_wav_are_refused(self):
        self.captions[1]['lines'] = ['different words.']
        with self.assertRaises(ValueError):
            self.run_retime()
        self.captions[1]['lines'] = ['three four.']
        self.timing['cues']['01_1']['wav_sha256'] = 'different'
        with self.assertRaises(ValueError):
            self.run_retime()

    def test_out_of_order_or_out_of_range_marks_are_refused(self):
        self.timing['cues']['01_1']['marks'] = [{'byte': 8, 'seconds': 4.0}]
        with self.assertRaises(ValueError):
            self.run_retime()
        self.timing['cues']['01_1']['marks'] = [{'byte': 8, 'seconds': 1.6}, {'byte': 4, 'seconds': 1.7}]
        with self.assertRaises(ValueError):
            self.run_retime()

    def test_sentence_boundary_does_not_move(self):
        text = 'One two. Three four.'
        self.script['01_1'] = text
        self.report['lines']['01_1'] = [
            {'sentence': 'One two.', 'out_samples': 48000, 'gap_after_samples': 0},
            {'sentence': 'Three four.', 'out_samples': 48000, 'gap_after_samples': 0},
        ]
        self.timing['cues']['01_1']['text_sha256'] = hashlib.sha256(text.encode()).hexdigest()
        self.timing['cues']['01_1']['marks'] = [{'byte': 9, 'seconds': 2.2}]
        self.captions[0]['lines'] = ['One two.']
        self.captions[1]['lines'] = ['Three four.']
        original = copy.deepcopy(self.captions)
        self.assertEqual(self.run_retime(), ([], []))
        self.assertEqual(self.captions, original)


if __name__ == '__main__':
    unittest.main()
