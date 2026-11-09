"""CC2 helper 475: supplemental frames from apportionment preview458, for review 470 F1 and review 467. Prepared only.

462 samples every silent hold and four frames of each cue. A picture that builds in stages during a long cue, and
holds each stage, is sampled only when one of those frames lands in it (review 470 F1). This helper takes one frame
of every such held picture from the same preview458 movies 462 extracted from, plus CC3 467's two frames 08 3450 and
5738. The frames are fixed in cc2-apportionment-supplement475-frames.json, pinned below; nothing is chosen at run
time.

How the list was made (`inventory` recomputes it): a held picture is a run of at least 60 frames (1 s) of
timeline444's own 64x36 render in which no pixel moves more than 2 of 255 levels from one frame to the next. It is
listed when no frame of 462's plan falls inside it, and its middle frame is taken. Each entry records how far its
picture is from the nearest picture 462 samples, at 64x36. This is a coarse proxy: it finds pictures 462 does not
show. It says nothing about how they look.

Modes (python3 -I -B cc2-apportionment-supplement475.py MODE):
  inventory [--print]   read-only. Needs a finished, successful timeline444. Recomputes the list from timeline444's
                        timelines, cue tables, layout rows and 64x36 movies with 462's own plan; exits 0 only if it
                        equals the pinned list. --print prints the recomputed list instead.
  plan                  read-only. Needs timeline444, score445, preview458 and pictures462 finished and successful,
                        with stable receipts. Every preview timeline must be byte-equal to timeline444's and to the
                        list's pin. Every preview movie must be the one 462 extracted from (its sampling.json hash)
                        and pass 462's probe. Every frame must be inside its chapter and not one 462 already took.
                        Prints the frames.
  inputs                after a successful plan: writes the guard input manifest once (never overwrites) and prints
                        the guard command. It launches nothing.
  extract               only as the child of the original guard (job apportionment-frederick-pictures475): rechecks
                        everything, then extracts the listed frames with 462's own extract_frames (one decode per
                        chapter, select=eq(n,...)) and 462's own 8-picture sheets, into the job folder.

It accepts no picture, motion, sound, fine-sync or AV result; the frames are material for a person to look at.
"""
import argparse
import hashlib
import importlib.util
import json
import subprocess
import sys
from pathlib import Path

VERSION = '475'
HELPER = Path(__file__).resolve()
HERE = HELPER.parent
H462 = HERE/'cc0-apportionment-preview-review462.py'
H462_SHA = '235755141df29ee093daf0af2a91a2ed6a6d0dfc5db74586c13193e78128bf37'
FRAMES = HERE/'cc2-apportionment-supplement475-frames.json'
FRAMES_SHA = 'de89f991660ac899572c8f8e4d306ca03429032dd8f9c2871f04d2d96e9943df'
STILL_LEVEL, HELD_FRAMES = 2, 60
SMALL = (64, 36)
FROM_467 = {'08_signposts': {3450: 'review 467: the last drain frame, the shortest think-ring arc (08-L2)',
                             5738: 'review 467: the "=" flash peak, nearest the right edge (08-L4)'}}
METHOD = ('held picture: >= %d frames of timeline444\'s 64x36 render with no pixel step > %d/255 between frames, and '
          'no frame of 462\'s plan inside; its middle frame is listed' % (HELD_FRAMES, STILL_LEVEL))


def load462():
    with H462.open('rb') as f:
        assert hashlib.file_digest(f, 'sha256').hexdigest() == H462_SHA, 'helper 462 changed'
    spec = importlib.util.spec_from_file_location('helper462_for475', H462)
    h = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(h)
    assert h.VERSION == '462' and h.HELPER == H462
    return h


def paths(h):
    return h.WORK/'apportionment-frederick-pictures475', h.S/'pictures475-inputs.json'


def chapter_inventory(h, c):
    """The held pictures of one chapter that 462's plan does not land in, from timeline444's own files."""
    import numpy as np
    files = {k: h.TIMELINE/(c+ext) for k, ext in (('timeline', '.timeline.tsv'), ('cues', '.cues.tsv'),
                                                  ('layout', '.layout.tsv'), ('movie', '.mp4'))}
    name, frames, lines, marks = h.read_timeline(files['timeline'])
    assert name == c, files['timeline']
    cues = h.read_cues(files['cues'], lines)
    anchors = h.merge_anchors([h.read_layout(files['layout'], 'timeline444')])
    taken = [s['frame'] for s in h.plan_chapter(c, frames, cues, marks, anchors)['samples']]
    raw = subprocess.run([str(h.FFMPEG), '-nostdin', '-v', 'error', '-xerror', '-threads', '1', '-i', str(files['movie']),
                          '-an', '-fps_mode', 'passthrough', '-f', 'rawvideo', '-pix_fmt', 'rgb24', '-'],
                         check=True, capture_output=True, timeout=1200).stdout
    assert len(raw) == frames*SMALL[0]*SMALL[1]*3, 'timeline444 movie %s is not %d frames at 64x36' % (c, frames)
    px = np.frombuffer(raw, np.uint8).reshape(frames, SMALL[1], SMALL[0], 3).astype(np.int16)
    step = np.abs(np.diff(px, axis=0)).max(axis=(1, 2, 3))      # step[k]: frame k to k+1
    rows, k = [], 0
    while k < len(step):
        if step[k] > STILL_LEVEL:
            k += 1
            continue
        j = k
        while j < len(step) and step[j] <= STILL_LEVEL:
            j += 1
        first, last, k = k, j, j                                  # frames first..last show one picture
        if last-first+1 < HELD_FRAMES or any(first <= f <= last for f in taken):
            continue
        level = int(np.abs(px[taken]-px[first]).reshape(len(taken), -1).max(axis=1).min())
        during = [n for n, s, e in cues if s <= first < e or s <= last < e or (first < s and last >= e)] or ['silent hold']
        rows.append(dict(frame=(first+last)//2, kind='held', first=first, last=last,
                         seconds=round((last-first+1)/h.FPS, 2), during=during, nearest_462_level=level,
                         reason='held %d-%d (%.2f s) during %s; no 462 frame inside; nearest 462 picture %d/255 away '
                                'at 64x36' % (first, last, (last-first+1)/h.FPS, '+'.join(during), level)))
    for f, why in FROM_467.get(c, {}).items():
        assert f not in taken and 0 <= f < frames, (c, f)
        rows.append(dict(frame=f, kind='467', reason=why))
    rows.sort(key=lambda r: r['frame'])
    assert len({r['frame'] for r in rows}) == len(rows), c
    pin = dict(frames=frames, plan462_samples=len(taken), **{k+'_sha256': h.sha(p) for k, p in files.items()})
    return rows, pin


def inventory(h):
    h.success(h.T444/'resource-receipt.json')
    summary = json.loads((h.T444/'timeline444-summary.json').read_text())
    chapters = summary['chapters']
    assert len(chapters) == len(set(chapters)) == 15
    out, pins = {}, {}
    for c in chapters:
        out[c], pins[c] = chapter_inventory(h, c)
        assert pins[c]['frames'] == summary['frames'][c], c
    return dict(version=VERSION, helper462_sha256=H462_SHA, method=METHOD, timeline444=pins, chapters=out,
                total=sum(len(r) for r in out.values()))


def build(h):
    """The frames per chapter, after every prerequisite check, and the files read."""
    assert FRAMES_SHA and h.sha(FRAMES) == FRAMES_SHA, 'frame list missing or changed'
    for path, digest in {**h.GUARD_CHAIN, **h.PREVIEW_HELPERS}.items():
        assert h.sha(path) == digest, 'pin changed: %s' % path
    listed = json.loads(FRAMES.read_text())
    assert listed['version'] == VERSION and listed['helper462_sha256'] == H462_SHA
    receipts = [h.T444/'resource-receipt.json', h.S445/'resource-receipt.json', h.P458/'resource-receipt.json',
                h.JOB/'resource-receipt.json']
    for r in receipts:
        h.success(r)
    h.stable(receipts)
    assert h.sha(h.S/'preview458-inputs.json') == json.loads(receipts[2].read_text())['inputs_sha256']
    assert h.sha(h.INPUTS) == json.loads(receipts[3].read_text())['inputs_sha256'], '462 ran from other inputs'
    p_summary = json.loads((h.P458/'preview458-summary.json').read_text())
    s462 = json.loads((h.JOB/'pictures462-summary.json').read_text())
    chapters = p_summary['chapters']
    assert p_summary['all_timelines_byte_identical'] is True and len(chapters) == 15
    assert chapters == list(listed['chapters']) == list(listed['timeline444'])
    assert s462['version'] == '462' and list(s462['chapters']) == chapters
    read = receipts + [h.S/'preview458-inputs.json', h.INPUTS, h.P458/'preview458-summary.json',
                       h.JOB/'pictures462-summary.json']
    plans = {}
    for c in chapters:
        pin, rows = listed['timeline444'][c], listed['chapters'][c]
        timeline = h.PREVIEW/(c+'.timeline.tsv')
        assert h.sha(timeline) == h.sha(h.TIMELINE/(c+'.timeline.tsv')) == pin['timeline_sha256'], 'timeline ' + c
        name, frames, _, _ = h.read_timeline(timeline)
        assert name == c and frames == pin['frames'], c
        sampling = h.JOB/c/'sampling.json'
        assert h.sha(sampling) == s462['chapters'][c]['sampling_sha256'], 'sampling ' + c
        done = json.loads(sampling.read_text())
        movie = h.PREVIEW/(c+'.mp4')
        assert done['movie'] == dict(path=str(movie), sha256=h.sha(movie)), 'not the movie 462 extracted from: ' + c
        read += [timeline, h.TIMELINE/(c+'.timeline.tsv'), sampling]
        if not rows:
            continue
        wanted = [r['frame'] for r in rows]
        assert wanted == sorted(set(wanted)) and all(0 <= f < frames for f in wanted), 'frames ' + c
        again = sorted(set(wanted) & {s['frame'] for s in done['samples']})
        assert not again, '462 already took %s in %s' % (again, c)
        h.probe(movie, frames)
        read.append(movie)
        plans[c] = dict(chapter=c, frames=frames, fps=h.FPS, movie=dict(path=str(movie), sha256=done['movie']['sha256']),
                        samples=[dict(frame=r['frame'], seconds=round(r['frame']/h.FPS, 4), kind=r['kind'],
                                      reasons=[r['reason']]) for r in rows])
    assert sum(len(p['samples']) for p in plans.values()) == listed['total']
    return plans, sorted(set(read))


def manifest_rows(h, read):
    out, seen = [], set()
    for p in [HELPER, FRAMES, H462, *h.GUARD_CHAIN, *h.PREVIEW_HELPERS, h.FFMPEG.resolve(), h.FFPROBE.resolve(), *read]:
        if str(p) not in seen:
            assert p.is_absolute() and not p.is_symlink(), p
            out.append(dict(path=str(p), sha256=h.sha(p)))
            seen.add(str(p))
    return out


def guard_command(h):
    job, inputs = paths(h)
    return [sys.executable, str(h.GUARD), '--phase', 'native', '--job', job.name, '--inputs', str(inputs), '--',
            sys.executable, '-I', '-B', str(HELPER), 'extract']


def extract(h):
    job, inputs = paths(h)
    receipt = json.loads((job/'resource-receipt.json').read_text())
    assert receipt.get('admitted') is True and 'finished' not in receipt and receipt.get('phase') == 'native', 'guard child only'
    assert receipt['command'][-4:] == ['-I', '-B', str(HELPER), 'extract'], receipt['command']
    assert receipt['inputs_sha256'] == h.sha(inputs), 'not the reviewed input manifest'
    assert sorted(p.name for p in job.iterdir()) == ['command.log', 'resource-receipt.json'], 'job folder not fresh'
    plans, read = build(h)
    bound = {r['path']: r['sha256'] for r in json.loads(inputs.read_text())['files']}
    for row in manifest_rows(h, read):
        assert bound.get(row['path']) == row['sha256'], 'not bound by the input manifest: '+row['path']
    done = {}
    for c, plan in plans.items():
        folder = job/c
        files = h.extract_frames(Path(plan['movie']['path']), [s['frame'] for s in plan['samples']], folder)
        for s, path in zip(plan['samples'], files):
            assert path.name == 'frame-%06d.jpg' % s['frame']
            s.update(path=str(path), sha256=h.sha(path))
        plan.update(sheets=h.sheets(c, plan['samples'], folder),
                    scope='Supplemental 540p preview458 frames for a person to look at; no picture, motion, sound, '
                          'fine-sync or AV acceptance.')
        with (folder/'sampling475.json').open('x') as f:
            json.dump(plan, f, indent=2)
            f.write('\n')
        done[c] = dict(samples=len(plan['samples']), sheets=len(plan['sheets']),
                       sampling_sha256=h.sha(folder/'sampling475.json'))
        h.say(chapter=c, **done[c])
    with (job/'pictures475-summary.json').open('x') as f:
        json.dump(dict(version=VERSION, frames_sha256=FRAMES_SHA, chapters=done,
                       total_samples=sum(d['samples'] for d in done.values()),
                       claims='None beyond the extraction itself.'), f, indent=2)
        f.write('\n')
    return 0


def main():
    assert sys.version_info >= (3, 11), 'needs hashlib.file_digest'
    h = load462()
    parser = argparse.ArgumentParser()
    modes = parser.add_subparsers(dest='mode', required=True)
    modes.add_parser('inventory').add_argument('--print', action='store_true')
    for m in ('plan', 'inputs', 'extract'):
        modes.add_parser(m)
    args = parser.parse_args()
    if args.mode == 'inventory':
        got = inventory(h)
        if args.print:
            print(json.dumps(got, indent=1))
            return 0
        assert FRAMES_SHA and h.sha(FRAMES) == FRAMES_SHA, 'frame list missing or changed'
        assert got == json.loads(FRAMES.read_text()), 'timeline444 no longer gives the pinned frame list'
        h.say(version=VERSION, inventory='equals the pinned frame list', total=got['total'],
              chapters={c: len(r) for c, r in got['chapters'].items()})
        return 0
    if args.mode == 'extract':
        return extract(h)
    plans, read = build(h)
    h.say(version=VERSION, chapters={c: len(p['samples']) for c, p in plans.items()},
          total=sum(len(p['samples']) for p in plans.values()))
    if args.mode == 'plan':
        for c, p in plans.items():
            for s in p['samples']:
                print('%s %6d %8.3f  %s' % (c, s['frame'], s['seconds'], s['reasons'][0]))
    if args.mode == 'inputs':
        job, inputs = paths(h)
        assert not inputs.exists() and not job.exists(), 'inputs or job already exist; never overwrite'
        rows = manifest_rows(h, read)
        with inputs.open('x') as f:
            json.dump(dict(files=rows), f, indent=2)
            f.write('\n')
        h.say(inputs=str(inputs), sha256=h.sha(inputs), files=len(rows), launched=False, guard_command=guard_command(h))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (AssertionError, OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(json.dumps(dict(refused='%s: %s' % (type(error).__name__, error), launched=False)), flush=True)
        raise SystemExit(1)
