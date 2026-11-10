#!/usr/bin/env python3
"""Extend the display of apportionment caption 168 by 20 ms in the SRT and the VTT.

Verifier 701 stopped on caption 168: 84 characters over 4180 ms is 20.0957 per second and its
limit is 20. This tool moves that one end from 616290 ms to 616310 ms, which is 84 characters
over 4200 ms, exactly 20, and the verifier fails only a rate above 20. Words, line breaks,
every start and every other end keep their exact bytes.

This is a display dwell adjustment. It is not evidence about when the words are spoken.

Both inputs must match the whole-file sha256 given on the command line and hold the exact
expected cue. Everything is checked before --out is made, and --out must not exist. The inputs
are only read. Exit 0 prints one JSON line, exit 1 means refused, exit 2 means bad arguments.
A write failure after --out is made leaves that directory in place; use a new name.
"""
import argparse
import hashlib
import json
import os
import re
import sys
from pathlib import Path

CUE = 168
START_MS = 612110
OLD_END_MS = 616290
NEW_END_MS = 616310
NEXT_START_MS = 616900
TEXT = 'Choose the number of people per seat that\nmakes the whole parts add up to the house.'

# The rules of apportionment-csd-verify701.py, which stays as it is.
LIMIT = 20
MAX_LINES = 2
MAX_LINE_CHARS = 42
TOTAL_FRAMES = 100886
FPS = 60

SRT_NAME = 'apportionment.en.srt'
VTT_NAME = 'apportionment.en.vtt'
SCOPE = 'display dwell adjustment only; not evidence of speech timing'


class Refusal(Exception):
    pass


def need(ok, why):
    if not ok:
        raise Refusal(why)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    try:
        return Path(path).read_bytes()
    except OSError as e:
        raise Refusal(f'cannot read input: {e.strerror}') from e


def clock(ms, decimal):
    return f'{ms // 3600000:02}:{ms // 60000 % 60:02}:{ms // 1000 % 60:02}{decimal}{ms % 1000:03}'


def stamp_ms(s):
    need(re.fullmatch(r'\d{2,}:\d{2}:\d{2}[.,]\d{3}', s), f'bad timestamp {s!r}')
    h, m, t = s.replace(',', '.').split(':')
    sec, ms = t.split('.')
    need(int(m) < 60 and int(sec) < 60, f'bad timestamp {s!r}')
    return ((int(h) * 60 + int(m)) * 60 + int(sec)) * 1000 + int(ms)


def parse(data, vtt):
    """The parse of captions() in verifier 701, refusing where it asserts."""
    kind = 'VTT' if vtt else 'SRT'
    need(not data.startswith(b'\xef\xbb\xbf'), f'{kind}: byte order mark')
    need(b'\r' not in data, f'{kind}: carriage return')
    try:
        text = data.decode('utf-8')
    except UnicodeDecodeError as e:
        raise Refusal(f'{kind}: not UTF-8') from e
    blocks = text.strip().split('\n\n')
    if vtt:
        need(blocks.pop(0) == 'WEBVTT', 'VTT: no WEBVTT header')
    film_end = round(TOTAL_FRAMES * 1000 / FPS)
    out = []
    for i, block in enumerate(blocks, 1):
        lines = block.splitlines()
        need(lines, f'{kind} cue {i}: empty')
        if '-->' not in lines[0]:
            need(lines.pop(0) == str(i), f'{kind} cue {i}: wrong index line')
            need(lines, f'{kind} cue {i}: no timing')
        times = lines.pop(0).split(' --> ')
        need(len(times) == 2, f'{kind} cue {i}: bad timing line')
        start, end = map(stamp_ms, times)
        need(0 <= start < end <= film_end, f'{kind} cue {i}: {start} to {end} is not inside the film')
        need(1 <= len(lines) <= MAX_LINES and all(0 < len(x) <= MAX_LINE_CHARS for x in lines),
             f'{kind} cue {i}: lines do not fit')
        need(not out or start >= out[-1]['end_ms'], f'{kind} cue {i}: overlaps the cue before it')
        out.append(dict(start_ms=start, end_ms=end, text='\n'.join(lines)))
    need(out, f'{kind}: no cues')
    return out


def fast_cues(caps):
    return [i for i, c in enumerate(caps, 1)
            if len(c['text'].replace('\n', ' ')) * 1000 / (c['end_ms'] - c['start_ms']) > LIMIT]


def exact_block(end_ms, srt):
    """The bytes of the cue with the start of the next one, as the kit writes them."""
    decimal = ',' if srt else '.'
    index, next_index = (f'{CUE}\n', f'{CUE + 1}\n') if srt else ('', '')
    return (f'\n\n{index}{clock(START_MS, decimal)} --> {clock(end_ms, decimal)}\n{TEXT}\n\n'
            f'{next_index}{clock(NEXT_START_MS, decimal)} --> ').encode('ascii')


def extend_cue(data, srt):
    kind = 'SRT' if srt else 'VTT'
    old = exact_block(OLD_END_MS, srt)
    found = data.count(old)
    need(found == 1, f'{kind}: the exact block for cue {CUE} occurs {found} times, expected once')
    out = data.replace(old, exact_block(NEW_END_MS, srt), 1)
    stamp = clock(OLD_END_MS, ',' if srt else '.').encode('ascii')
    hundreds = data.index(old) + old.index(stamp) + len(stamp) - 3
    changed = [i for i, (a, b) in enumerate(zip(data, out)) if a != b]
    need(len(out) == len(data) and changed == [hundreds, hundreds + 1], f'{kind}: the edit reached other bytes')
    return out


def repair(srt, vtt):
    before = parse(srt, vtt=False)
    need(before == parse(vtt, vtt=True), 'the SRT and the VTT do not hold the same cues')
    need(len(before) > CUE, f'{len(before)} cues found, cue {CUE + 1} is required')
    need(before[CUE - 1] == dict(start_ms=START_MS, end_ms=OLD_END_MS, text=TEXT),
         f'cue {CUE} is not the expected cue: {before[CUE - 1]}')
    need(before[CUE]['start_ms'] == NEXT_START_MS,
         f'cue {CUE + 1} starts at {before[CUE]["start_ms"]}, expected {NEXT_START_MS}')
    fast = fast_cues(before)
    need(fast == [CUE], f'fast cues are {fast}, expected only [{CUE}]')

    new_srt, new_vtt = extend_cue(srt, srt=True), extend_cue(vtt, srt=False)

    after = parse(new_srt, vtt=False)
    need(after == parse(new_vtt, vtt=True), 'the repaired SRT and VTT do not hold the same cues')
    expected = [dict(c) for c in before]
    expected[CUE - 1]['end_ms'] = NEW_END_MS
    need(after == expected, f'the output differs from the input beyond the end of cue {CUE}')
    need(not fast_cues(after), f'the output still has fast cues: {fast_cues(after)}')
    return new_srt, new_vtt, len(before)


def run(srt_path, vtt_path, srt_sha256, vtt_sha256, out_dir):
    for name, value in (('--srt-sha256', srt_sha256), ('--vtt-sha256', vtt_sha256)):
        need(re.fullmatch('[0-9a-f]{64}', value), f'{name} is not a lowercase sha256')
    srt, vtt = read(srt_path), read(vtt_path)
    need(digest(srt) == srt_sha256, f'the SRT is {digest(srt)}, expected {srt_sha256}')
    need(digest(vtt) == vtt_sha256, f'the VTT is {digest(vtt)}, expected {vtt_sha256}')
    new_srt, new_vtt, cues = repair(srt, vtt)

    out = Path(out_dir)
    need(not os.path.lexists(out), 'the output directory already exists')
    need(out.parent.is_dir(), 'the parent of the output directory does not exist')
    os.mkdir(out)
    for name, data in ((SRT_NAME, new_srt), (VTT_NAME, new_vtt)):
        with open(out / name, 'xb') as f:
            f.write(data)
        need(read(out / name) == data, f'{name}: read back differs')

    chars = len(TEXT.replace('\n', ' '))
    return dict(cue=CUE, cues=cues, characters=chars, start_ms=START_MS, end_ms_before=OLD_END_MS, end_ms_after=NEW_END_MS,
                next_start_ms=NEXT_START_MS, gap_after_ms=NEXT_START_MS - NEW_END_MS,
                chars_per_second_before=chars * 1000 / (OLD_END_MS - START_MS),
                chars_per_second_after=chars * 1000 / (NEW_END_MS - START_MS), limit=LIMIT,
                input_srt_sha256=srt_sha256, input_vtt_sha256=vtt_sha256,
                output_srt_sha256=digest(new_srt), output_vtt_sha256=digest(new_vtt), scope=SCOPE)


def main():
    parser = argparse.ArgumentParser(description='Extend the display of apportionment caption 168 by 20 ms.')
    parser.add_argument('--srt', required=True, help='the SRT the master wrote; only read')
    parser.add_argument('--vtt', required=True, help='the VTT the master wrote; only read')
    parser.add_argument('--srt-sha256', required=True, help='expected sha256 of the whole SRT')
    parser.add_argument('--vtt-sha256', required=True, help='expected sha256 of the whole VTT')
    parser.add_argument('--out', required=True, help='new directory for the repaired pair; must not exist')
    args = parser.parse_args()
    try:
        report = run(args.srt, args.vtt, args.srt_sha256, args.vtt_sha256, args.out)
    except (Refusal, OSError) as e:
        print(f'refused: {e}', file=sys.stderr)
        return 1
    print(json.dumps(report, sort_keys=True))
    return 0


if __name__ == '__main__':
    sys.exit(main())
