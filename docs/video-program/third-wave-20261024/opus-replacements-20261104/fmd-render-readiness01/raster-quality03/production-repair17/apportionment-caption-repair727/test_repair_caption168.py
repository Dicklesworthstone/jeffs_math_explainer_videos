#!/usr/bin/env python3
"""Tests for repair_caption168.py. Run: python3 -I -B test_repair_caption168.py

The fixture is synthetic, in the byte format of to_srt and to_vtt in the kit's captions.rs.
Cue 168 and the start of cue 169 carry the real values. No actual master captions are used.
Temporary files are left in the system temporary directory; nothing here deletes a file.
"""
import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.dont_write_bytecode = True

HERE = Path(__file__).resolve().parent
HELPER = HERE / 'repair_caption168.py'
VERIFIER = HERE.parent / 'apportionment-csd-verify701.py'
VERIFIER_SHA256 = '02ef7a5f4aae336b4a3795b066352a85c9a536d953c8920fab4755a2f174417f'
FAST_LINE = (r"fast = [dict(index=i+1, **c) for i, c in enumerate(caps) "
             r"if len(c['text'].replace('\n', ' '))*1000/(c['end_ms']-c['start_ms']) > 20]")

START, OLD_END, NEW_END, NEXT_START = 612110, 616290, 616310, 616900
TEXT = 'Choose the number of people per seat that\nmakes the whole parts add up to the house.'
FILM_FRAMES = 100886


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


repair = load('repair_caption168', HELPER)
verifier = load('verify701', VERIFIER)
SCRATCH = Path(tempfile.mkdtemp(prefix='caption727-'))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def clock(ms, decimal):
    return f'{ms // 3600000:02}:{ms // 60000 % 60:02}:{ms // 1000 % 60:02}{decimal}{ms % 1000:03}'


def cues():
    out, t = [], 600
    for i in range(1, 168):
        lines = [f'Synthetic caption {i} on one line.'] if i % 2 else [f'Synthetic caption {i} is', 'shown on two short lines.']
        out.append(dict(start=t, end=t + 3000, lines=lines))
        t += 3600
    out.append(dict(start=START, end=OLD_END, lines=TEXT.split('\n')))
    t = NEXT_START
    for i in range(169, 181):
        out.append(dict(start=t, end=t + 3000, lines=[f'Synthetic caption {i} after it.']))
        t += 3600
    return out


def srt_of(items):
    return ''.join(f"{k}\n{clock(c['start'], ',')} --> {clock(c['end'], ',')}\n" + '\n'.join(c['lines']) + '\n\n'
                   for k, c in enumerate(items, 1)).encode()


def vtt_of(items):
    return ('WEBVTT\n\n' + ''.join(f"{clock(c['start'], '.')} --> {clock(c['end'], '.')}\n" + '\n'.join(c['lines']) + '\n\n'
                                   for c in items)).encode()


def mutated(edit):
    items = cues()
    edit(items)
    return srt_of(items), vtt_of(items)


def real_fast(caps):
    namespace = {'caps': caps}
    exec(FAST_LINE, namespace)
    return namespace['fast']


class Case(unittest.TestCase):
    def setUp(self):
        self.dir = Path(tempfile.mkdtemp(prefix='t', dir=SCRATCH))
        self.out = self.dir / 'out'

    def write(self, srt, vtt):
        a, b = self.dir / 'in.srt', self.dir / 'in.vtt'
        a.write_bytes(srt)
        b.write_bytes(vtt)
        return a, b

    def repaired(self):
        srt, vtt = srt_of(cues()), vtt_of(cues())
        a, b = self.write(srt, vtt)
        report = repair.run(a, b, sha(srt), sha(vtt), self.out)
        return srt, vtt, (self.out / repair.SRT_NAME), (self.out / repair.VTT_NAME), report

    def refused(self, srt, vtt, pattern):
        a, b = self.write(srt, vtt)
        with self.assertRaisesRegex(repair.Refusal, pattern):
            repair.run(a, b, sha(srt), sha(vtt), self.out)
        self.assertFalse(os.path.lexists(self.out), 'a refusal must not make the output directory')
        self.assertEqual((a.read_bytes(), b.read_bytes()), (srt, vtt))


class Pins(unittest.TestCase):
    def test_the_helper_holds_the_task_values(self):
        self.assertEqual((repair.CUE, repair.START_MS, repair.OLD_END_MS, repair.NEW_END_MS, repair.NEXT_START_MS),
                         (168, START, OLD_END, NEW_END, NEXT_START))
        self.assertEqual(repair.TEXT, TEXT)
        self.assertEqual([len(x) for x in TEXT.split('\n')], [41, 42])
        self.assertEqual(len(TEXT.replace('\n', ' ')), 84)

    def test_the_verifier_is_the_pinned_one_with_its_limit_unchanged(self):
        source = VERIFIER.read_bytes()
        self.assertEqual(sha(source), VERIFIER_SHA256)
        self.assertEqual([x.strip() for x in source.decode().splitlines() if x.strip().startswith('fast = [')], [FAST_LINE])
        self.assertEqual(verifier.TOTAL_FRAMES, FILM_FRAMES)
        self.assertEqual((repair.TOTAL_FRAMES, repair.LIMIT, repair.MAX_LINE_CHARS, repair.MAX_LINES), (FILM_FRAMES, 20, 42, 2))

    def test_the_stamps_are_the_expected_text(self):
        self.assertEqual(repair.clock(START, ','), '00:10:12,110')
        self.assertEqual(repair.clock(OLD_END, ','), '00:10:16,290')
        self.assertEqual(repair.clock(NEW_END, '.'), '00:10:16.310')
        self.assertEqual(repair.clock(NEXT_START, '.'), '00:10:16.900')


class ExactChange(Case):
    def test_output_equals_an_independent_rebuild(self):
        _, _, srt_out, vtt_out, _ = self.repaired()
        self.assertEqual(srt_out.read_bytes(), mutated(lambda c: c[167].update(end=NEW_END))[0])
        self.assertEqual(vtt_out.read_bytes(), mutated(lambda c: c[167].update(end=NEW_END))[1])

    def test_only_the_two_digits_of_that_end_change(self):
        srt, vtt, srt_out, vtt_out, _ = self.repaired()
        for before, after, old, new in ((srt, srt_out.read_bytes(), b'00:10:16,290', b'00:10:16,310'),
                                        (vtt, vtt_out.read_bytes(), b'00:10:16.290', b'00:10:16.310')):
            self.assertEqual(len(before), len(after))
            changed = [i for i, (a, b) in enumerate(zip(before, after)) if a != b]
            self.assertEqual(len(changed), 2)
            at = before.index(old)
            self.assertEqual(before.count(old), 1)
            self.assertEqual(changed, [at + 9, at + 10])
            self.assertEqual(after[at:at + len(new)], new)

    def test_inputs_stay_as_they_were(self):
        srt, vtt, _, _, _ = self.repaired()
        self.assertEqual((self.dir / 'in.srt').read_bytes(), srt)
        self.assertEqual((self.dir / 'in.vtt').read_bytes(), vtt)

    def test_the_report_names_the_change_and_its_scope(self):
        _, _, srt_out, vtt_out, report = self.repaired()
        self.assertEqual(report['output_srt_sha256'], sha(srt_out.read_bytes()))
        self.assertEqual(report['output_vtt_sha256'], sha(vtt_out.read_bytes()))
        self.assertEqual((report['end_ms_before'], report['end_ms_after'], report['gap_after_ms']), (OLD_END, NEW_END, 590))
        self.assertEqual(report['chars_per_second_after'], 20.0)
        self.assertEqual(round(report['chars_per_second_before'], 4), 20.0957)
        self.assertIn('not evidence of speech timing', report['scope'])

    def test_words_breaks_and_all_other_timings_are_kept(self):
        srt, _, srt_out, _, _ = self.repaired()
        before = repair.parse(srt, vtt=False)
        after = repair.parse(srt_out.read_bytes(), vtt=False)
        self.assertEqual([c['text'] for c in before], [c['text'] for c in after])
        self.assertEqual([c['start_ms'] for c in before], [c['start_ms'] for c in after])
        self.assertEqual([i for i, (x, y) in enumerate(zip(before, after), 1) if x['end_ms'] != y['end_ms']], [168])


class NoOverlapNoShortening(Case):
    def test_nothing_overlaps_and_the_gap_is_590(self):
        _, _, srt_out, _, _ = self.repaired()
        caps = repair.parse(srt_out.read_bytes(), vtt=False)
        self.assertTrue(all(b['start_ms'] >= a['end_ms'] for a, b in zip(caps, caps[1:])))
        self.assertEqual(caps[168]['start_ms'] - caps[167]['end_ms'], 590)

    def test_no_cue_is_shorter_and_only_168_grows_by_20(self):
        srt, _, srt_out, _, _ = self.repaired()
        before = repair.parse(srt, vtt=False)
        after = repair.parse(srt_out.read_bytes(), vtt=False)
        growth = [(b['end_ms'] - b['start_ms']) - (a['end_ms'] - a['start_ms']) for a, b in zip(before, after)]
        self.assertTrue(all(x >= 0 for x in growth))
        self.assertEqual(growth[167], 20)
        self.assertEqual(sum(growth), 20)


class Threshold(Case):
    def test_the_input_reproduces_the_reported_failure(self):
        a, b = self.write(srt_of(cues()), vtt_of(cues()))
        caps = verifier.captions(a, FILM_FRAMES)
        self.assertEqual(caps, verifier.captions(b, FILM_FRAMES, True))
        self.assertEqual(real_fast(caps), [dict(index=168, start_ms=START, end_ms=OLD_END, text=TEXT)])

    def test_the_real_verifier_reads_the_output_as_passing(self):
        srt, _, srt_out, vtt_out, _ = self.repaired()
        caps = verifier.captions(srt_out, FILM_FRAMES)
        self.assertEqual(caps, verifier.captions(vtt_out, FILM_FRAMES, True))
        self.assertEqual(real_fast(caps), [])
        words = lambda items: ' '.join(' '.join(c['text'].split()) for c in items)
        self.assertEqual(words(caps), words(repair.parse(srt, vtt=False)))

    def test_the_new_end_is_exactly_on_the_limit(self):
        self.assertEqual(84 * 1000, 20 * (NEW_END - START))
        self.assertFalse(84 * 1000 / (NEW_END - START) > 20)

    def test_one_millisecond_less_would_still_fail(self):
        caps = [dict(start_ms=START, end_ms=NEW_END - 1, text=TEXT)]
        self.assertEqual(len(real_fast(caps)), 1)

    def test_the_helper_and_the_verifier_agree_on_fast_cues(self):
        for end in (OLD_END, NEW_END - 1, NEW_END, NEW_END + 1):
            caps = [dict(start_ms=START, end_ms=end, text=TEXT)]
            self.assertEqual(repair.fast_cues(caps), [x['index'] for x in real_fast(caps)])


class Refusals(Case):
    def test_wrong_hashes(self):
        srt, vtt = srt_of(cues()), vtt_of(cues())
        a, b = self.write(srt, vtt)
        for args, pattern in (((sha(b'x'), sha(vtt)), 'the SRT is'), ((sha(srt), sha(b'x')), 'the VTT is'),
                              ((sha(srt).upper(), sha(vtt)), 'not a lowercase'), (('', sha(vtt)), 'not a lowercase')):
            with self.assertRaisesRegex(repair.Refusal, pattern):
                repair.run(a, b, *args, self.out)
            self.assertFalse(os.path.lexists(self.out))

    def test_a_changed_word(self):
        self.refused(*mutated(lambda c: c[167].update(lines=['Choose the count of people per seat that', c[167]['lines'][1]])), 'cue 168 is not the expected')

    def test_a_moved_line_break(self):
        self.refused(*mutated(lambda c: c[167].update(lines=['Choose the number of people per seat', 'that makes the whole parts add up to the house.'])), 'SRT cue 168: lines do not fit')

    def test_a_moved_start(self):
        self.refused(*mutated(lambda c: c[167].update(start=START + 1)), 'cue 168 is not the expected')

    def test_a_moved_end(self):
        self.refused(*mutated(lambda c: c[167].update(end=OLD_END + 1)), 'cue 168 is not the expected')

    def test_an_already_repaired_input(self):
        self.refused(*mutated(lambda c: c[167].update(end=NEW_END)), 'cue 168 is not the expected')

    def test_a_shifted_next_cue(self):
        self.refused(*mutated(lambda c: c[168].update(start=NEXT_START + 5)), 'cue 169 starts at 616905')

    def test_a_next_cue_the_new_end_would_overlap(self):
        self.refused(*mutated(lambda c: c[168].update(start=NEW_END - 10)), 'cue 169 starts at 616300')

    def test_a_timing_shift_of_the_whole_film(self):
        def shift(items):
            for c in items:
                c['start'] += 17
                c['end'] += 17
        self.refused(*mutated(shift), 'cue 168 is not the expected')

    def test_a_cue_added_before_it(self):
        self.refused(*mutated(lambda c: c.insert(0, dict(start=0, end=500, lines=['Extra.']))), 'cue 168 is not the expected')

    def test_a_cue_removed_before_it(self):
        self.refused(*mutated(lambda c: c.pop(0)), 'cue 168 is not the expected')

    def test_the_cue_missing(self):
        self.refused(*mutated(lambda c: c.pop(167)), 'cue 168 is not the expected')

    def test_no_next_cue(self):
        self.refused(*mutated(lambda c: c.__delitem__(slice(168, None))), 'cue 169 is required')

    def test_a_second_fast_cue(self):
        self.refused(*mutated(lambda c: c[10].update(end=c[10]['start'] + 1000)), r'fast cues are \[11, 168\]')

    def test_a_repeated_cue_is_an_overlap(self):
        self.refused(*mutated(lambda c: c.insert(168, dict(c[167]))), 'SRT cue 169: overlaps')

    def test_a_repeated_exact_block(self):
        block = repair.exact_block(OLD_END, True)
        with self.assertRaisesRegex(repair.Refusal, 'occurs 2 times'):
            repair.extend_cue(b'1' + block + b'x\n' + block + b'y', srt=True)

    def test_srt_and_vtt_that_differ(self):
        srt = srt_of(cues())
        _, vtt = mutated(lambda c: c[3].update(end=c[3]['end'] + 1))
        self.refused(srt, vtt, 'do not hold the same')

    def test_crlf(self):
        srt, vtt = srt_of(cues()), vtt_of(cues())
        self.refused(srt.replace(b'\n', b'\r\n'), vtt, 'SRT: carriage return')
        self.refused(srt, vtt.replace(b'\n', b'\r\n'), 'VTT: carriage return')

    def test_bom(self):
        srt, vtt = srt_of(cues()), vtt_of(cues())
        self.refused(b'\xef\xbb\xbf' + srt, vtt, 'SRT: byte order mark')
        self.refused(srt, b'\xef\xbb\xbf' + vtt, 'VTT: byte order mark')

    def test_not_utf8(self):
        srt, vtt = srt_of(cues()), vtt_of(cues())
        self.refused(srt.replace(b'Synthetic caption 2 ', b'Synthetic caption \xff '), vtt, 'SRT: not UTF-8')

    def test_no_webvtt_header(self):
        self.refused(srt_of(cues()), vtt_of(cues())[len(b'WEBVTT\n\n'):], 'no WEBVTT header')

    def test_a_wrong_srt_index(self):
        srt = srt_of(cues()).replace(b'\n\n12\n', b'\n\n13\n', 1)
        self.refused(srt, vtt_of(cues()), 'SRT cue 12: wrong index line')

    def test_an_overlap_elsewhere(self):
        self.refused(*mutated(lambda c: c[20].update(end=c[21]['start'] + 1)), 'SRT cue 22: overlaps')

    def test_a_long_line_elsewhere(self):
        self.refused(*mutated(lambda c: c[20].update(lines=['x' * 43])), 'SRT cue 21: lines do not fit')

    def test_three_lines_elsewhere(self):
        self.refused(*mutated(lambda c: c[20].update(lines=['a', 'b', 'c'])), 'SRT cue 21: lines do not fit')

    def test_an_end_past_the_film(self):
        self.refused(*mutated(lambda c: c[-1].update(end=1681434)), 'is not inside the film')

    def test_a_bad_timestamp(self):
        srt = srt_of(cues()).replace(b'00:10:12,110', b'0:10:12,110')
        self.refused(srt, vtt_of(cues()), 'bad timestamp')


class ExactBytes(Case):
    """Inputs the parser accepts and reads as equal, refused only by the exact match."""

    def test_an_unpadded_or_longer_hour_field(self):
        srt = srt_of(cues()).replace(b'00:10:12,110', b'000:10:12,110')
        self.refused(srt, vtt_of(cues()), 'SRT: the exact block for cue 168 occurs 0 times')

    def test_a_comma_in_the_vtt_stamp(self):
        vtt = vtt_of(cues()).replace(b'00:10:12.110 --> 00:10:16.290', b'00:10:12,110 --> 00:10:16,290')
        self.refused(srt_of(cues()), vtt, 'VTT: the exact block for cue 168 occurs 0 times')

    def test_an_index_line_in_the_vtt(self):
        vtt = vtt_of(cues()).replace(b'\n\n00:10:12.110', b'\n\n168\n00:10:12.110')
        self.refused(srt_of(cues()), vtt, 'VTT: the exact block for cue 168 occurs 0 times')

    def test_no_index_line_in_the_srt_cue(self):
        srt = srt_of(cues()).replace(b'\n\n168\n00:10:12,110', b'\n\n00:10:12,110')
        self.refused(srt, vtt_of(cues()), 'SRT: the exact block for cue 168 occurs 0 times')


class PostConditions(Case):
    """A faulty edit is refused by the checks after it, before anything is written."""

    def attempt(self, pattern):
        self.refused(srt_of(cues()), vtt_of(cues()), pattern)

    def test_an_edit_that_reaches_another_cue(self):
        wider = lambda data, srt: data.replace(b'Synthetic caption 1 on', b'Synthetic caption 9 on')
        with mock.patch.object(repair, 'extend_cue', wider):
            self.attempt('beyond the end of cue 168')

    def test_an_edit_that_reaches_other_bytes_of_the_block(self):
        real = repair.exact_block

        def wider(end_ms, srt):
            block = real(end_ms, srt)
            return block.replace(b'Choose', b'Chooze') if end_ms == NEW_END else block

        with mock.patch.object(repair, 'exact_block', wider):
            self.attempt('the edit reached other bytes')

    def test_an_edit_of_one_file_only(self):
        real = repair.extend_cue
        with mock.patch.object(repair, 'extend_cue', lambda data, srt: real(data, srt) if srt else data):
            self.attempt('repaired SRT and VTT do not hold the same')

    def test_an_end_short_of_the_limit(self):
        with mock.patch.object(repair, 'NEW_END_MS', NEW_END - 10):
            self.attempt('still has fast cues')


class Output(Case):
    def attempt(self, out, pattern):
        srt, vtt = srt_of(cues()), vtt_of(cues())
        a, b = self.write(srt, vtt)
        with self.assertRaisesRegex(repair.Refusal, pattern):
            repair.run(a, b, sha(srt), sha(vtt), out)
        self.assertEqual((a.read_bytes(), b.read_bytes()), (srt, vtt))

    def test_an_existing_directory(self):
        self.out.mkdir()
        self.attempt(self.out, 'already exists')
        self.assertEqual(list(self.out.iterdir()), [])

    def test_the_input_directory(self):
        self.attempt(self.dir, 'already exists')

    def test_an_input_file_as_output(self):
        self.attempt(self.dir / 'in.srt', 'already exists')

    def test_a_missing_parent(self):
        self.attempt(self.dir / 'missing' / 'out', 'parent')

    def test_a_file_that_appears_in_the_new_directory_is_not_overwritten(self):
        srt, vtt = srt_of(cues()), vtt_of(cues())
        a, b = self.write(srt, vtt)
        real = os.mkdir

        def racing(path):
            real(path)
            (Path(path) / 'apportionment.en.srt').write_bytes(b'written by someone else')

        with mock.patch.object(repair.os, 'mkdir', racing):
            with self.assertRaises(FileExistsError):
                repair.run(a, b, sha(srt), sha(vtt), self.out)
        self.assertEqual((self.out / 'apportionment.en.srt').read_bytes(), b'written by someone else')

    def test_a_missing_input(self):
        with self.assertRaisesRegex(repair.Refusal, 'cannot read input'):
            repair.run(self.dir / 'absent.srt', self.dir / 'absent.vtt', sha(b''), sha(b''), self.out)
        self.assertFalse(os.path.lexists(self.out))


class ParserParity(Case):
    """The embedded parser and the verifier's own accept and reject the same bytes."""

    def bad_srt(self):
        good = cues()
        yield mutated(lambda c: c[20].update(end=c[21]['start'] + 1))[0]
        yield mutated(lambda c: c[20].update(lines=['x' * 43]))[0]
        yield mutated(lambda c: c[20].update(lines=['a', 'b', 'c']))[0]
        yield mutated(lambda c: c[20].update(end=c[20]['start']))[0]
        yield mutated(lambda c: c[-1].update(end=1681434))[0]
        yield srt_of(good).replace(b'\n\n12\n', b'\n\n13\n', 1)
        yield srt_of(good).replace(b'00:10:12,110', b'0:10:12,110')
        yield srt_of(good).replace(b'00:10:12,110', b'00:60:12,110')
        yield srt_of(good).replace(b'00:10:12,110', b'00:10:60,110')

    def test_same_cues_on_good_input_and_output(self):
        srt, vtt, srt_out, vtt_out, _ = self.repaired()
        self.assertEqual(verifier.captions(self.dir / 'in.srt', FILM_FRAMES), repair.parse(srt, vtt=False))
        self.assertEqual(verifier.captions(self.dir / 'in.vtt', FILM_FRAMES, True), repair.parse(vtt, vtt=True))
        self.assertEqual(verifier.captions(srt_out, FILM_FRAMES), repair.parse(srt_out.read_bytes(), vtt=False))
        self.assertEqual(verifier.captions(vtt_out, FILM_FRAMES, True), repair.parse(vtt_out.read_bytes(), vtt=True))

    def test_both_reject_the_same_bad_files(self):
        path = self.dir / 'bad.srt'
        cases = list(self.bad_srt())
        self.assertEqual(len(cases), 9)
        for bad in cases:
            path.write_bytes(bad)
            with self.assertRaises(repair.Refusal):
                repair.parse(bad, vtt=False)
            with self.assertRaises((AssertionError, ValueError, IndexError)):
                verifier.captions(path, FILM_FRAMES)

    def test_both_reject_a_vtt_without_its_header(self):
        path = self.dir / 'bad.vtt'
        bad = vtt_of(cues())[len(b'WEBVTT\n\n'):]
        path.write_bytes(bad)
        with self.assertRaises(repair.Refusal):
            repair.parse(bad, vtt=True)
        with self.assertRaises(AssertionError):
            verifier.captions(path, FILM_FRAMES, True)


class CommandLine(Case):
    def call(self, *args):
        return subprocess.run([sys.executable, '-I', '-B', str(HELPER), *map(str, args)],
                              capture_output=True, text=True, timeout=60, check=False)

    def test_success_prints_one_json_line(self):
        srt, vtt = srt_of(cues()), vtt_of(cues())
        a, b = self.write(srt, vtt)
        done = self.call('--srt', a, '--vtt', b, '--srt-sha256', sha(srt), '--vtt-sha256', sha(vtt), '--out', self.out)
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(len(done.stdout.splitlines()), 1)
        report = json.loads(done.stdout)
        self.assertEqual(report['output_srt_sha256'], sha((self.out / 'apportionment.en.srt').read_bytes()))
        self.assertEqual(report['output_vtt_sha256'], sha((self.out / 'apportionment.en.vtt').read_bytes()))

    def test_refusal_exits_1_and_prints_no_report(self):
        srt, vtt = srt_of(cues()), vtt_of(cues())
        a, b = self.write(srt, vtt)
        done = self.call('--srt', a, '--vtt', b, '--srt-sha256', sha(b'x'), '--vtt-sha256', sha(vtt), '--out', self.out)
        self.assertEqual((done.returncode, done.stdout), (1, ''))
        self.assertIn('refused:', done.stderr)
        self.assertFalse(os.path.lexists(self.out))

    def test_every_argument_is_required(self):
        self.assertEqual(self.call('--srt', 'a').returncode, 2)
        self.assertFalse(os.path.lexists(self.out))


if __name__ == '__main__':
    unittest.main()
