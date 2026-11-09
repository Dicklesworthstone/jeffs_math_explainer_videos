"""CC0 helper 462: picture samples from the finished apportionment preview458, for review. Prepared only.

Every sample frame is derived from the actual produced files: each chapter's timeline (frames, cue starts and
`mark predict` frames), cue table (audio lengths) and layout lint rows (preview458's own and timeline444's). Nothing
is assumed about a chapter that its files do not say. A frame picked for several reasons keeps all of them.

Per chapter it samples:
  - the first and last frame;
  - every cue: start, +0.5 s, middle and last spoken frame (start + audio length - 1 frame);
  - every hold (lead-in, each gap between cues, the end hold): first, middle and last frame;
  - every prediction hold (a predict mark between a cue's end and the next cue, as captions.rs reads it): the mark,
    +0.3 s (the "?" has faded in), +2.0 s (PREDICT_HOLD_S), the last frame before the next cue, that cue's start and
    +0.5 s;
  - every layout lint row (anchor): its first look, +0.2 s (the next sampled look), +1.0 s (the persist window, so
    the flag stood here), +2.0 s, every 2 s of the chapter grid after that through the chapter end, and the end of
    the span the "later looks" would cover if they were consecutive sampled looks (they need not be: the lint also
    counts checkpoint looks, and a flag may clear and return), plus 1 s after it.

Chapter 08 must carry the three anchors at 0.8 s, 53.4 s and 93.2 s that timeline444 reported.

Modes (python3 -I -B cc0-apportionment-preview-review462.py MODE):
  plan [--show CHAPTER]   read-only. Requires timeline444, score445 and preview458 finished and successful, stable
                          receipts, 15 chapters, every preview timeline byte-equal to timeline444's, and each preview
                          movie 960x540 at 60 fps with as many packets as its timeline has frames. Prints the plan.
  plan --dry-run-timeline444 [--show CHAPTER]
                          read-only, for review of the sampling only: plans from timeline444's files present now. No
                          preview, receipt or movie is checked and nothing is claimed.
  inputs                  after a successful plan: writes the guard input manifest once (refuses to overwrite) and
                          prints the guard command. It launches nothing.
  extract                 only as the child of the original guard (guard_owner80g_target20g.py, job
                          apportionment-frederick-pictures462): rechecks everything, then extracts the planned frames
                          with ffmpeg into the job folder, with one sampling.json and 8-picture sheets per chapter.

It accepts no picture, motion, sound, fine-sync or AV result; the frames are material for a person to look at.
"""
import argparse
import hashlib
import json
import math
import re
import subprocess
import sys
import time
from pathlib import Path

VERSION = '462'
HELPER = Path(__file__).resolve()
HERE = HELPER.parent
REPO = HERE.parents[6]
WORK = REPO/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
S = REPO/'narration_lab/third-wave-20261024/root/elevenlabs-apportionment-rebuild-20261104/frederick-generation89'
T444, S445, P458 = (WORK/('apportionment-frederick-'+n) for n in ('timeline444', 'score445', 'preview458'))
TIMELINE, PREVIEW = T444/'timeline', P458/'preview'
JOB = WORK/'apportionment-frederick-pictures462'
INPUTS = S/'pictures462-inputs.json'
GUARD = HERE/'guard_owner80g_target20g.py'
GUARD_CHAIN = {GUARD: '4f34e0d1e9a8f9d164860828f87a734e99b697d2d3048c7b65508e5d84b8b4ad',
               REPO/'docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/'
                    'guard_owner56g_target20g.py': '1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097',
               REPO/'narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/'
                    'run_bounded_phase_oliver_owner48g_target14g_20261102.py':
                   '819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da'}
PREVIEW_HELPERS = {S/'preview458.py': '7ab178d17b2e0279ccf5940d9327dc691610c676e855228ed5391ae5be1c073d',
                   S/'preview-after-score458.py': '3a7c4b0c41474412b654de6dd83137a820bf361611d9c53c162cb286d829b805'}
FFMPEG, FFPROBE = Path('/opt/homebrew/bin/ffmpeg'), Path('/opt/homebrew/bin/ffprobe')
FPS, SIZE = 60, (960, 540)
SAMPLE_S, PERSIST_S, PREDICT_HOLD_S = 0.2, 1.0, 2.0      # explainer_kit layout.rs SAMPLE_S/PERSIST_S, props.rs
GRID_S = 2.0
ANCHORS08 = (0.8, 53.4, 93.2)
STABLE_SECONDS = 15
MAX_SAMPLES = 2500


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def say(**fields):
    print(json.dumps(fields), flush=True)


def success(receipt_path):
    d = json.loads(receipt_path.read_text())
    assert d.get('finished') and d.get('admitted') is True and d.get('preflight_only') is False, receipt_path
    assert d.get('exit') == 0 and not any(d.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), \
        'not a successful terminal receipt: %s' % receipt_path
    assert d['inputs_sha256'] == d['final_inputs_sha256'], receipt_path
    return d


def stable(paths):
    first = {p: sha(p) for p in paths}
    time.sleep(STABLE_SECONDS)
    assert first == {p: sha(p) for p in paths}, 'a receipt changed between two reads'


def read_timeline(path):
    rows = [line.split('\t') for line in path.read_text().splitlines() if line and not line.startswith('#')]
    head = {r[0]: r[1] for r in rows if r[0] in ('chapter', 'fps', 'frames')}
    assert set(head) == {'chapter', 'fps', 'frames'} and int(head['fps']) == FPS, path
    assert not [r[0] for r in rows if r[0] not in ('chapter', 'fps', 'frames', 'line', 'mark')], path
    lines = [(r[1], int(r[2])) for r in rows if r[0] == 'line']
    marks = [(r[1], int(r[2])) for r in rows if r[0] == 'mark']
    return head['chapter'], int(head['frames']), lines, marks


def read_cues(path, lines):
    rows = [line.split('\t') for line in path.read_text().splitlines() if line.strip()]
    assert [r[0] for r in rows] == [cue for cue, _ in lines], 'cue table and timeline name different cues: %s' % path
    out = []
    for (cue, start), r in zip(lines, rows):
        assert round(float(r[1])*FPS) == start, 'cue %s starts at %s s, timeline frame %d' % (cue, r[1], start)
        out.append((cue, start, round((float(r[1])+float(r[2]))*FPS)))     # end frame is exclusive
    return out


def read_layout(path, source):
    anchors = []
    for row in path.read_text().splitlines():
        if not row.strip():
            continue
        at, kind, detail = row.split('\t', 2)
        found = re.findall(r'@(\d+(?:\.\d+)?)s', at)
        assert found, 'layout row without a time: %r' % row
        later = re.search(r'\(still there at (\d+) later looks\)$', detail)
        anchors.append(dict(row=row, sources=[source], seconds=float(found[-1]), kind=kind,
                            later_looks=int(later.group(1)) if later else 0))
    return anchors


def plan_chapter(chapter, frames, cues, marks, anchors):
    picks, skipped = {}, []

    def add(frame, reason):
        if 0 <= frame < frames:
            reasons = picks.setdefault(frame, [])
            if reason not in reasons:
                reasons.append(reason)
        else:
            skipped.append(dict(frame=frame, reason=reason))

    add(0, 'chapter first frame')
    add(frames-1, 'chapter last frame')
    holds = []
    for k, (cue, start, end) in enumerate(cues):
        add(start, 'cue %s start' % cue)
        add(start+30, 'cue %s +0.5 s' % cue)
        add((start+end)//2, 'cue %s middle' % cue)
        add(min(end, frames)-1, 'cue %s last spoken frame' % cue)
        after = cues[k+1][1] if k+1 < len(cues) else frames
        holds.append(('hold after %s' % cue, end, after, cues[k+1][0] if k+1 < len(cues) else None))
    if cues:
        holds.insert(0, ('lead-in before %s' % cues[0][0], 0, cues[0][1], cues[0][0]))
    for name, lo, hi, nxt in holds:
        if hi - lo < 1:
            skipped.append(dict(frame=lo, reason='%s: no hold (%d frames)' % (name, hi-lo)))
            continue
        add(lo, '%s: first frame' % name)
        add((lo+hi)//2, '%s: middle (%.2f s hold)' % (name, (hi-lo)/FPS))
        add(hi-1, '%s: last frame%s' % (name, ' before '+nxt if nxt else ''))
    predicts = []
    for label, frame in marks:
        if label != 'predict':
            add(frame, 'mark %s' % label)
            continue
        k = next((k for k, (_, _, end) in enumerate(cues) if end <= frame and (k+1 == len(cues) or frame < cues[k+1][1])), None)
        nxt = cues[k+1] if k is not None and k+1 < len(cues) else None
        tag = 'predict %d' % frame
        predicts.append(dict(frame=frame, after=cues[k][0] if k is not None else None, reveal=nxt[0] if nxt else None,
                             in_a_hold=nxt is not None))
        add(frame, tag+': mark')
        add(frame+18, tag+': +0.3 s ("?" faded in)')
        add(frame+round(PREDICT_HOLD_S*FPS), tag+': +2.0 s (PREDICT_HOLD_S)')
        if nxt:
            add(nxt[1]-1, tag+': last frame before %s' % nxt[0])
            add(nxt[1], tag+': %s starts (answer may show)' % nxt[0])
            add(nxt[1]+30, tag+': %s +0.5 s' % nxt[0])
    for n, a in enumerate(anchors, 1):
        a['id'] = '%s-L%d' % (chapter[:2], n)
        f0 = round(a['seconds']*FPS)
        tag = '%s %s @%.1fs' % (a['id'], a['kind'], a['seconds'])
        add(f0, tag+': first look')
        add(f0+round(SAMPLE_S*FPS), tag+': next sampled look (+0.2 s)')
        add(f0+round(PERSIST_S*FPS), tag+': +1.0 s (stood through the persist window)')
        add(f0+2*FPS, tag+': +2.0 s')
        step = round(GRID_S*FPS)
        for f in range(math.ceil((f0+round(PERSIST_S*FPS))/step)*step, frames, step):
            add(f, tag+': 2 s grid through chapter end')
        span = a['seconds'] + PERSIST_S + SAMPLE_S*a['later_looks']
        a['consecutive_span_end_s'] = round(span, 3)
        if round(span*FPS) < frames:
            add(round(span*FPS), tag+': end if the %d later looks were consecutive' % a['later_looks'])
            add(round(span*FPS)+FPS, tag+': 1 s after that end')
        add(frames-1, tag+': chapter last frame')
    return dict(samples=[dict(frame=f, seconds=round(f/FPS, 4), reasons=picks[f]) for f in sorted(picks)],
                skipped=skipped, holds=[dict(name=h[0], first=h[1], end=h[2]) for h in holds], predicts=predicts,
                anchors=anchors)


def merge_anchors(groups):
    out = {}
    for anchors in groups:
        for a in anchors:
            if a['row'] in out:
                out[a['row']]['sources'] += a['sources']
            else:
                out[a['row']] = a
    return sorted(out.values(), key=lambda a: (a['seconds'], a['kind'], a['row']))


def probe(movie, frames):
    got = json.loads(subprocess.run([str(FFPROBE), '-v', 'error', '-select_streams', 'v:0', '-count_packets',
                                     '-show_entries', 'stream=width,height,avg_frame_rate,nb_read_packets', '-of', 'json',
                                     str(movie)], check=True, capture_output=True, text=True, timeout=600).stdout)['streams'][0]
    assert (got['width'], got['height']) == SIZE and got['avg_frame_rate'] == '%d/1' % FPS, (movie, got)
    assert int(got['nb_read_packets']) == frames, 'movie %s has %s packets, timeline %d frames' % (
        movie, got['nb_read_packets'], frames)


def build(dry_run=False):
    """The plan for every chapter, from the files as they are now, and the files it read."""
    read = []
    if dry_run:
        chapters = sorted(p.name.removesuffix('.timeline.tsv') for p in TIMELINE.glob('*.timeline.tsv'))
        source = TIMELINE
    else:
        for path, digest in {**GUARD_CHAIN, **PREVIEW_HELPERS}.items():
            assert sha(path) == digest, 'pin changed: %s' % path
        receipts = [T444/'resource-receipt.json', S445/'resource-receipt.json', P458/'resource-receipt.json']
        for r in receipts:
            success(r)
        stable(receipts)
        assert sha(S/'preview458-inputs.json') == json.loads(receipts[2].read_text())['inputs_sha256']
        summary = json.loads((P458/'preview458-summary.json').read_text())
        chapters = summary['chapters']
        assert summary['all_timelines_byte_identical'] is True and len(chapters) == len(set(chapters)) == 15
        t_summary = json.loads((T444/'timeline444-summary.json').read_text())
        assert t_summary['chapters'] == chapters
        read += receipts + [S/'preview458-inputs.json', P458/'preview458-summary.json', T444/'timeline444-summary.json']
        source = PREVIEW
        assert {p.name for p in PREVIEW.glob('*.timeline.tsv')} == {c+'.timeline.tsv' for c in chapters}
    plans, offset = {}, 0
    for c in chapters:
        name, frames, lines, marks = read_timeline(source/(c+'.timeline.tsv'))
        assert name == c
        cues = read_cues(source/(c+'.cues.tsv'), lines)
        groups = [read_layout(TIMELINE/(c+'.layout.tsv'), 'timeline444')]
        read += [source/(c+'.timeline.tsv'), source/(c+'.cues.tsv'), TIMELINE/(c+'.layout.tsv')]
        if not dry_run:
            assert (PREVIEW/(c+'.timeline.tsv')).read_bytes() == (TIMELINE/(c+'.timeline.tsv')).read_bytes(), c
            assert t_summary['frames'][c] == frames, c
            groups.append(read_layout(PREVIEW/(c+'.layout.tsv'), 'preview458'))
            probe(PREVIEW/(c+'.mp4'), frames)
            read += [TIMELINE/(c+'.timeline.tsv'), PREVIEW/(c+'.layout.tsv'), PREVIEW/(c+'.mp4')]
        anchors = merge_anchors(groups)
        if c == '08_signposts':
            times = {a['seconds'] for a in anchors}
            assert all(t in times for t in ANCHORS08), '08 anchors %s missing from %s' % (ANCHORS08, sorted(times))
        plans[c] = dict(chapter=c, frames=frames, fps=FPS, registry_offset_frames=offset, **plan_chapter(c, frames, cues, marks, anchors))
        offset += frames
    total = sum(len(p['samples']) for p in plans.values())
    assert total <= MAX_SAMPLES, 'plan has %d samples, more than %d' % (total, MAX_SAMPLES)
    return plans, sorted(set(read))


def manifest_rows(read):
    files = [HELPER, *GUARD_CHAIN, *PREVIEW_HELPERS, FFMPEG.resolve(), FFPROBE.resolve(), *read]
    files += sorted(p for p in P458.rglob('*') if p.is_file() and not p.is_symlink())
    out, seen = [], set()
    for p in files:
        if str(p) not in seen:
            assert p.is_absolute() and not p.is_symlink(), p
            out.append(dict(path=str(p), sha256=sha(p)))
            seen.add(str(p))
    return out


def guard_command():
    return [sys.executable, str(GUARD), '--phase', 'native', '--job', JOB.name, '--inputs', str(INPUTS), '--',
            sys.executable, '-I', '-B', str(HELPER), 'extract']


def balanced(xs):
    if len(xs) == 1:
        return xs[0]
    k = len(xs)//2
    return '('+balanced(xs[:k])+'+'+balanced(xs[k:])+')'


def extract_frames(movie, frames, folder):
    """ffmpeg decodes the movie once and writes exactly the selected frame indices, in order, as JPEG."""
    folder.mkdir(parents=True)
    script = folder/'select.filter'
    with script.open('x') as f:
        f.write("select='"+balanced(['eq(n,%d)' % n for n in frames])+"'")
    seq = folder/'seq'
    seq.mkdir()
    subprocess.run([str(FFMPEG), '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2', '-i', str(movie), '-an',
                    '-/filter:v', str(script), '-fps_mode', 'passthrough', '-q:v', '2', str(seq/'seq-%06d.jpg')],
                   check=True, timeout=1200)
    made = sorted(seq.glob('seq-*.jpg'))
    assert len(made) == len(frames), 'ffmpeg wrote %d pictures for %d frames' % (len(made), len(frames))
    out = []
    for n, path in zip(frames, made):
        target = folder/('frame-%06d.jpg' % n)
        assert not target.exists()
        path.rename(target)
        out.append(target)
    return out


def sheets(chapter, samples, folder):
    from PIL import Image, ImageDraw, ImageFont
    try:
        font = ImageFont.load_default(size=15)
    except (TypeError, OSError):
        font = ImageFont.load_default()
    out = []
    for k in range(0, len(samples), 8):
        canvas = Image.new('RGB', (2*SIZE[0], 4*(SIZE[1]+46)), '#10151d')
        draw = ImageDraw.Draw(canvas)
        for i, s in enumerate(samples[k:k+8]):
            x, y = (i % 2)*SIZE[0], (i//2)*(SIZE[1]+46)
            with Image.open(s['path']) as im:
                assert im.size == SIZE, s['path']
                canvas.paste(im.convert('RGB'), (x, y))
            draw.text((x+6, y+SIZE[1]+3), '%s frame %d  %.3f s' % (chapter, s['frame'], s['seconds']), fill='white', font=font)
            draw.text((x+6, y+SIZE[1]+23), '; '.join(s['reasons'])[:118], fill='#c8d2e0', font=font)
        path = folder/('sheet-%04d.jpg' % (k//8+1))
        assert not path.exists()
        canvas.save(path, quality=92)
        out.append(dict(path=str(path), sha256=sha(path)))
    return out


def extract():
    receipt = json.loads((JOB/'resource-receipt.json').read_text())
    assert receipt.get('admitted') is True and 'finished' not in receipt and receipt.get('phase') == 'native', 'guard child only'
    assert receipt['command'][-4:] == ['-I', '-B', str(HELPER), 'extract'], receipt['command']
    assert receipt['inputs_sha256'] == sha(INPUTS), 'not the reviewed input manifest'
    assert sorted(p.name for p in JOB.iterdir()) == ['command.log', 'resource-receipt.json'], 'job folder not fresh'
    plans, read = build()
    bound = {r['path']: r['sha256'] for r in json.loads(INPUTS.read_text())['files']}
    for row in manifest_rows(read):
        assert bound.get(row['path']) == row['sha256'], 'not bound by the input manifest: '+row['path']
    done = {}
    for c, plan in plans.items():
        folder = JOB/c
        movie = PREVIEW/(c+'.mp4')
        files = extract_frames(movie, [s['frame'] for s in plan['samples']], folder)
        for s, path in zip(plan['samples'], files):
            s.update(path=str(path), sha256=sha(path))
        plan.update(movie=dict(path=str(movie), sha256=sha(movie)), sheets=sheets(c, plan['samples'], folder),
                    scope='Selected 540p preview458 frames for a person to look at; no picture, motion, sound, '
                          'fine-sync or AV acceptance.')
        with (folder/'sampling.json').open('x') as f:
            json.dump(plan, f, indent=2)
            f.write('\n')
        done[c] = dict(samples=len(plan['samples']), sheets=len(plan['sheets']), sampling_sha256=sha(folder/'sampling.json'))
        say(chapter=c, **done[c])
    with (JOB/'pictures462-summary.json').open('x') as f:
        json.dump(dict(version=VERSION, chapters=done, total_samples=sum(d['samples'] for d in done.values()),
                       claims='None beyond the extraction itself.'), f, indent=2)
        f.write('\n')
    return 0


def main():
    assert sys.version_info >= (3, 11), 'needs hashlib.file_digest'
    assert (REPO/'explainer_corpus/apportionment/script.tsv').is_file(), 'helper is not at its Q path'
    parser = argparse.ArgumentParser()
    modes = parser.add_subparsers(dest='mode', required=True)
    plan = modes.add_parser('plan')
    plan.add_argument('--dry-run-timeline444', action='store_true')
    plan.add_argument('--show')
    modes.add_parser('inputs')
    modes.add_parser('extract')
    args = parser.parse_args()
    if args.mode == 'extract':
        return extract()
    plans, read = build(dry_run=args.mode == 'plan' and args.dry_run_timeline444)
    say(version=VERSION, dry_run=args.mode == 'plan' and args.dry_run_timeline444, chapters={
        c: dict(frames=p['frames'], samples=len(p['samples']), anchors=len(p['anchors']), predicts=len(p['predicts']),
                skipped=len(p['skipped'])) for c, p in plans.items()}, total_samples=sum(len(p['samples']) for p in plans.values()))
    if args.mode == 'plan' and args.show:
        p = plans[args.show]
        for a in p['anchors']:
            say(anchor=a['id'], seconds=a['seconds'], kind=a['kind'], later_looks=a['later_looks'], sources=a['sources'],
                consecutive_span_end_s=a['consecutive_span_end_s'])
        for s in p['samples']:
            print('%6d %8.3f  %s' % (s['frame'], s['seconds'], ' | '.join(s['reasons'])))
        for s in p['skipped']:
            say(skipped=s)
    if args.mode == 'inputs':
        assert not INPUTS.exists() and not JOB.exists(), 'inputs or job already exist; never overwrite'
        rows = manifest_rows(read)
        with INPUTS.open('x') as f:
            json.dump(dict(files=rows), f, indent=2)
            f.write('\n')
        say(inputs=str(INPUTS), sha256=sha(INPUTS), files=len(rows), launched=False, guard_command=guard_command())
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except AssertionError as error:
        say(refused=str(error), launched=False)
        raise SystemExit(1)
