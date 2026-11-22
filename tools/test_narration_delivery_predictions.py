"""Prediction-hold windows from synthetic timeline rows; no audio, timing or listening claim."""
from collections import Counter
import unittest
import narration_delivery as nd

FPS = 30
ORDER = ['07_1', '07_2', '08_1', '08_2']
# Every synthetic cue lasts exactly two seconds of 24 kHz audio.
OUTPUTS = {cue: {'frames': 48000, 'sample_rate': 24000} for cue in ORDER}


def line(cue, frame):
    return ['line', cue, str(frame), '0', f'/{cue}.wav']


CHAPTER = [line('07_1', 0), line('07_2', 150)]   # 07_2 audio runs 5 s to 7 s
FRAMES = 330                                     # chapter ends at 11 s


def holds(predicts, declared, chapter_end=frozenset(), offset=10.0):
    return nd.chapter_prediction_holds(CHAPTER, predicts, FPS, FRAMES, OUTPUTS, Counter(declared),
                                       set(chapter_end), ORDER, offset)


class PredictionHoldTests(unittest.TestCase):
    def test_in_chapter_pair_is_unchanged(self):
        first, last = holds([3.0], {('07_1', '07_2'): 1})
        self.assertEqual(first[1:], ('07_2', 10.0 + 5.0))
        self.assertEqual(first[2], nd.prediction_end(CHAPTER[0], CHAPTER[1], FPS, [3.0], OUTPUTS,
                                                     Counter({('07_1', '07_2'): 1}), 10.0))
        self.assertEqual(last[1:], ('08_1', None))

    def test_flagged_chapter_end_pair_holds_to_the_actual_final_frame(self):
        pair = ('07_2', '08_1')
        last = holds([8.0], {pair: 1}, {pair})[-1]
        self.assertEqual(last[1:], ('08_1', 10.0 + FRAMES / FPS))

    def test_chapter_end_window_is_cue_end_to_final_frame(self):
        pair = ('07_2', '08_1')
        for predicts in ([], [6.9], [FRAMES / FPS], [12.0]):
            with self.subTest(predicts=predicts):
                self.assertIsNone(holds(predicts, {pair: 1}, {pair})[-1][2])
        self.assertIsNotNone(holds([7.0], {pair: 1}, {pair})[-1][2])

    def test_unflagged_or_duplicate_cross_chapter_pair_keeps_legacy_none(self):
        pair = ('07_2', '08_1')
        self.assertIsNone(holds([8.0], {pair: 1})[-1][2])
        self.assertIsNone(holds([8.0], {pair: 2}, {pair})[-1][2])

    def test_chapter_end_flag_must_name_adjacent_chapter_boundary(self):
        def hold(cue, next_cue, **extra):
            return dict(video='t', cue=cue, next_cue=next_cue, kind='prediction', **extra)
        ok = {'visual_holds': [hold('07_1', '07_2'), hold('07_2', '08_1', chapter_end=True)]}
        declared, flagged = nd.declared_prediction_pairs(ok, 't', ORDER)
        self.assertEqual((declared, flagged), (Counter({('07_1', '07_2'): 1, ('07_2', '08_1'): 1}), {('07_2', '08_1')}))
        for wrong in (hold('07_1', '07_2', chapter_end=True), hold('07_1', '08_1', chapter_end=True),
                      hold('07_2', '09_1', chapter_end=True)):
            with self.subTest(wrong=wrong), self.assertRaises(ValueError):
                nd.declared_prediction_pairs({'visual_holds': [wrong]}, 't', ORDER)
        legacy = {'visual_holds': [hold('07_2', '08_1')]}
        self.assertEqual(nd.declared_prediction_pairs(legacy, 't', ORDER)[1], set())

    def test_required_policy_refuses_a_silently_dropped_pair(self):
        declared = Counter({('07_1', '07_2'): 1, ('07_2', '08_1'): 1})
        found = [{'cue': '07_1', 'next_cue': '07_2'}]
        nd.require_declared_predictions({}, declared, found, ORDER)
        with self.assertRaises(ValueError):
            nd.require_declared_predictions({'prediction_holds': 'required'}, declared, found, ORDER)
        with self.assertRaises(ValueError):
            nd.require_declared_predictions({'prediction_holds': 'optional'}, declared, found, ORDER)
        nd.require_declared_predictions({'prediction_holds': 'required'}, declared,
                                        found + [{'cue': '07_2', 'next_cue': '08_1'}], ORDER)
        # Only pairs whose held cue is selected are wanted.
        nd.require_declared_predictions({'prediction_holds': 'required'}, declared, found, ['07_1'])


if __name__ == '__main__':
    unittest.main()
