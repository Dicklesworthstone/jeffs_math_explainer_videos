"""Guarded CSD extraction of exact selected frames from one successful 4K final672 chapter (03-15). Prepared for root.

Run only on the CSD, with /usr/bin/python3 -I -B:
  preflight          read-only: stable successful import683, all 62 seed bindings at their imported physical paths and
                     hashes, and python, ffmpeg, ffprobe and Pillow exercised. Writes nothing.
  inputs CHAPTER     as preflight, plus every check on that chapter's final; then writes one new guard manifest.
  extract CHAPTER    only as the child of guard_csd02: rechecks everything, then writes the frames, sheets and records
                     inside the guard's new job folder.

Reuse: root's support616 (pinned) supplies success, stable, read_timeline, read_cues, read_layout, plan_chapter, probe,
extract_frames and sheets. Its one import-time path, REPO = HERE.parents[6], cannot resolve at a CSD path (the 668
failure class), so that single line is replaced in memory with the explicit assembly repo. No file of root's is edited.

Seed plan 677 keeps every sample and reason. Every row of the final's own layout.tsv is added through plan_chapter, so
each new lint flag gets its first look, persistence, grid and span samples.

It accepts no picture, motion, sound, fine-sync, AV or release result; root views the frames afterwards.
"""
import hashlib
import io
import json
import os
import re
import subprocess
import sys
import time
import types
from pathlib import Path

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
JOBS = D/'jobs'
A, B, T = JOBS/'csd-apportionment-assembly664', JOBS/'csd-apportionment-build656', JOBS/'csd-apportionment-timeline666'
IMPORT = JOBS/'csd-apportionment-import683-a1'
HELPER = Path(__file__).resolve()
CONTROLLER = Q/'apportionment-pictures-after-final679.py'
REL = 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
NARRATION = 'apportionment-picture-assembly566/narration'
EXPORT_SHA = '1c7c690b603ed0a3571ba27e033dc2c1af581cacf74b171e01996e702e2285c8'
PLAN_SHA = '8c15820840d9e99bc56e012705439ae19089099439af84ca6c2ee3448920211b'
SUPPORT_SHA = '4deb8cf2b846607c1a4733494c502c2d1ae0834ddfb59e46361f2bce69d9d112'
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4, work_cap_gib=56, target_growth_cap_gib=20,
              sample_seconds=10)
PYTHON = '/usr/bin/python3'
SCOPE = ('Exact selected frames from the 4K final672 chapter, not yet viewed. No motion, sound, fine-sync, mixed-host '
         'master, AV or release acceptance.')


def read(path):
    return json.loads(Path(path).read_text())


def live(path):
    """The guard rewrites its running receipt in place every 10 s, so a torn read is retried."""
    for _ in range(20):
        try:
            return read(path)
        except json.JSONDecodeError:
            time.sleep(0.1)
    raise SystemExit('the running receipt could not be read: %s' % path)


def one(rows, tail, digest):
    found = [r for r in rows if r['path'].endswith(tail)]
    assert len(found) == 1 and found[0]['sha256'] == digest, tail
    return found[0]


def load_support(rows):
    """Support616 from its imported bytes, authenticated by the pin, with its one import-time path made explicit."""
    path = Path(one(rows, '/root-apportionment-picture-support616.py', SUPPORT_SHA)['physical'])
    data = path.read_bytes()
    assert hashlib.sha256(data).hexdigest() == SUPPORT_SHA, path
    text, line = data.decode(), 'REPO = HERE.parents[6]'
    assert text.count(line) == 1, 'support616 no longer has the expected path line'
    h = types.ModuleType('support616_csd')
    h.__file__ = str(path)
    exec(compile(text.replace(line, 'REPO = Path(%r)' % str(A/'repo')), str(path), 'exec'), h.__dict__)
    h.FFMPEG, h.FFPROBE = Path('/usr/bin/ffmpeg'), Path('/usr/bin/ffprobe')
    return h, path


def setup():
    """Stable successful import683, then the seed plan and its 62 bindings at their imported physical paths."""
    receipt, result_path = IMPORT/'resource-receipt.json', IMPORT/'import-result.json'
    first = read(result_path)
    h, support = load_support(first['files']+first['reused_files'])
    h.success(receipt)
    h.stable([receipt, result_path])
    result = read(result_path)
    assert result['manifest_sha256'] == EXPORT_SHA, 'import683 carried another manifest'
    rows = result['files']+result['reused_files']
    mapping = {r['path']: r for r in rows}
    assert len(mapping) == len(rows) == 4005, len(rows)
    plan_path = Path(one(rows, '/apportionment-picture-plan677.json', PLAN_SHA)['physical'])
    assert h.sha(plan_path) == PLAN_SHA, plan_path
    plan = read(plan_path)
    local = plan['bindings'][0]['path'].split('docs/video-program')[0]
    assert local.endswith('/') and len(plan['bindings']) == 62 and len(plan['chapters']) == 15
    bound = {}
    for b in plan['bindings']:
        assert b['path'].startswith(local) and mapping[b['path']]['sha256'] == b['sha256'], b['path']
        p = Path(mapping[b['path']]['physical'])
        assert not p.is_symlink() and h.sha(p) == b['sha256'], b['path']
        bound[b['path'].removeprefix(local)] = p
    files = {receipt, result_path, plan_path, support, *bound.values()}
    return types.SimpleNamespace(h=h, plan=plan, bound=bound, local=local, files=files)


def libraries(path):
    """The shared libraries a binary or extension loads, as ldd resolves them; an unresolved one refuses."""
    out = subprocess.check_output(['/usr/bin/ldd', str(path)], text=True, timeout=60, env=dict(PATH='/usr/bin:/bin'))
    assert 'not found' not in out, path
    return {Path(m[1]).resolve() for m in (re.search(r'(/\S+)', line) for line in out.splitlines()) if m}


def runtime(h):
    """Python, ffmpeg, ffprobe and Pillow, exercised as sheets() uses them. Returns the files to bind and the record."""
    python = Path(sys.executable).resolve()
    assert python == Path(PYTHON).resolve(), 'run with '+PYTHON
    tools = {}
    for name, tool in (('ffmpeg', h.FFMPEG), ('ffprobe', h.FFPROBE)):
        line = subprocess.check_output([str(tool), '-version'], text=True, timeout=60).splitlines()[0]
        tools[name] = dict(path=str(Path(tool).resolve()), version=line)
    import PIL
    from PIL import Image, ImageDraw, ImageFont
    image = Image.new('RGB', (1920, 1080), '#10151d')
    ImageDraw.Draw(image).text((8, 8), 'frame 1  0.017 s', fill='white', font=ImageFont.load_default(size=15))
    stream = io.BytesIO()
    image.resize((960, 540), Image.Resampling.LANCZOS).save(stream, format='JPEG', quality=92)
    stream.seek(0)
    with Image.open(stream) as decoded:
        decoded.load()
        assert decoded.size == (960, 540), decoded.size
    folder = Path(PIL.__file__).parent
    pillow = {p.resolve() for p in folder.rglob('*') if p.is_file() and '__pycache__' not in p.parts}
    natives = sorted(p for p in pillow if p.suffix == '.so')
    assert natives, 'no native Pillow extensions found'
    files = {python, *pillow}
    for p in [python, Path(h.FFMPEG).resolve(), Path(h.FFPROBE).resolve(), *natives]:
        files |= {p} | libraries(p)
    return files, dict(python=str(python), python_version=sys.version, **tools, pillow=PIL.__version__,
                       pillow_package=str(folder))


def merge(h, seed, chapter, frames, cues, marks, layout):
    """The seed's samples and reasons, all kept, plus plan_chapter over the seed's anchors and every final lint row."""
    anchors = [dict(a) for a in seed['anchors']]
    known = {a['row']: a for a in anchors}
    final, new = h.read_layout(layout, 'final672'), []
    for a in final:
        if a['row'] not in known:
            known[a['row']] = a
            new.append(a)
        elif 'final672' not in known[a['row']]['sources']:
            known[a['row']]['sources'] = known[a['row']]['sources']+['final672']
    fresh = h.plan_chapter(chapter, frames, cues, marks, anchors+new)
    seed_ids = [a['id'] for a in seed['anchors']]
    assert [a['id'] for a in fresh['anchors'][:len(seed_ids)]] == seed_ids, (chapter, 'seed anchor ids moved')
    picks = {s['frame']: dict(frame=s['frame'], seconds=s['seconds'], reasons=list(s['reasons'])) for s in seed['samples']}
    for s in fresh['samples']:
        entry = picks.setdefault(s['frame'], dict(frame=s['frame'], seconds=s['seconds'], reasons=[]))
        entry['reasons'] += [r for r in s['reasons'] if r not in entry['reasons']]
    for s in seed['samples']:
        assert set(s['reasons']) <= set(picks[s['frame']]['reasons']), (chapter, s['frame'])
    samples = [picks[f] for f in sorted(picks)]
    assert samples and all(0 <= s['frame'] < frames for s in samples) and len(samples) <= h.MAX_SAMPLES, chapter
    return dict(samples=samples, holds=fresh['holds'], predicts=fresh['predicts'], anchors=fresh['anchors'],
                skipped=seed['skipped']+[x for x in fresh['skipped'] if x not in seed['skipped']],
                merge=dict(seed_samples=len(seed['samples']), merged_samples=len(samples), final_layout_rows=len(final),
                           final_rows_not_in_seed=len(new)))


def build(s, c):
    """Every check on one finished chapter. Returns its merged plan, its movie and every file to bind."""
    h = s.h
    chapters = read(B/'qualification.json')['chapters']
    assert len(chapters) == 15 and set(s.plan['chapters']) == set(chapters), 'chapter sets differ'
    assert c in chapters[2:], 'chapters 01 and 02 are preserved and are not extracted here'
    final = JOBS/('csd-apportionment-final672-'+c)
    receipt, summary_path, render = final/'resource-receipt.json', final/'final672-summary.json', final/'render'
    original = Q/('csd-apportionment-final672-'+c+'-inputs.json')
    for p in (A, B, T, final):
        h.success(p/'resource-receipt.json')
    h.stable([receipt, summary_path])
    r = read(receipt)
    assert r['command'][-2:] == [str(Q/'apportionment-csd-final672.py'), c], r['command']
    assert h.sha(original) == r['inputs_sha256'], original
    frozen = read(original)['files']
    for row in frozen:
        assert h.sha(row['path']) == row['sha256'], row['path']
    summary = read(summary_path)
    record = summary['result']
    assert summary['chapter'] == c and record['folder'] == str(render.relative_to(D)), c
    files = {HELPER, CONTROLLER, original, receipt, summary_path, T/'comparison666.json', B/'qualification.json', *s.files}
    files.update(Path(row['path']) for row in frozen)
    for name, digest in record['files'].items():
        assert not Path(name).is_absolute() and '..' not in Path(name).parts, name
        p = render/name
        assert not p.is_symlink() and h.sha(p) == digest, p
        files.add(p)
    seed, frames = s.plan['chapters'][c], record['frames']
    assert frames == seed['frames'] == read(T/'comparison666.json')['chapters'][c]['frames'], (c, frames)
    name, counted, lines, marks = h.read_timeline(render/(c+'.timeline.tsv'))
    assert name == c and counted == frames, (c, name, counted)
    here, there = str(A/'repo'/REL/NARRATION), s.local+REL+'/'+NARRATION
    tables = REL+'/apportionment-picture-timeline567/timeline/'
    for ext in ('timeline.tsv', 'cues.tsv'):
        made = (render/(c+'.'+ext)).read_text().replace(here, there)
        assert made == s.bound[tables+c+'.'+ext].read_text(), (c, ext, 'differs from bound timeline567')
    cues = h.read_cues(render/(c+'.cues.tsv'), lines)
    movie = render/(c+'.mp4')
    h.probe(movie, frames)
    plan = dict(chapter=c, frames=frames, fps=h.FPS, registry_offset_frames=seed['registry_offset_frames'],
                **merge(h, seed, c, frames, cues, marks, render/(c+'.layout.tsv')))
    tools, provenance = runtime(h)
    files |= tools
    plan.update(runtime=provenance, seed_plan_sha256=PLAN_SHA, final=dict(job=final.name, receipt_sha256=h.sha(receipt),
                summary_sha256=h.sha(summary_path), inputs_sha256=r['inputs_sha256']))
    return plan, movie, sorted(files)


def extract(s, c, job, inputs):
    h = s.h
    r = live(job/'resource-receipt.json')
    assert r.get('admitted') is True and r.get('phase') == 'native' and r.get('preflight_only') is False, 'guard child only'
    assert 'finished' not in r and 'exit' not in r and r['policy'] == POLICY, r.get('policy')
    assert r['command'] == [PYTHON, '-I', '-B', str(HELPER), 'extract', c], r['command']
    assert r['inputs_sha256'] == h.sha(inputs), 'not the frozen manifest'
    assert sorted(p.name for p in job.iterdir()) == ['command.log', 'resource-receipt.json'], 'job folder not fresh'
    assert os.getpgid(0) == os.getpid() == os.getsid(0), 'run as the guard command itself'
    os.sched_setaffinity(0, {0, 1, 2, 3})
    plan, movie, files = build(s, c)
    bound = {x['path']: x['sha256'] for x in read(inputs)['files']}
    for p in files:
        assert bound.get(str(p)) == h.sha(p), p
    folder = job/c
    pictures = h.extract_frames(movie, [x['frame'] for x in plan['samples']], folder)
    assert len(pictures) == len(plan['samples']), c
    for sample, path in zip(plan['samples'], pictures):
        sample.update(path=str(path), sha256=h.sha(path))
    plan.update(movie=dict(path=str(movie), sha256=h.sha(movie)), sheets=h.sheets(c, plan['samples'], folder), scope=SCOPE)
    with (folder/'sampling.json').open('x') as f:
        json.dump(plan, f, indent=2)
        f.write('\n')
    done = dict(chapter=c, frames=plan['frames'], samples=len(plan['samples']), sheets=len(plan['sheets']),
                sampling_sha256=h.sha(folder/'sampling.json'), movie_sha256=plan['movie']['sha256'], merge=plan['merge'],
                runtime=plan['runtime'], scope=SCOPE)
    with (job/'pictures678-summary.json').open('x') as f:
        json.dump(done, f, indent=2)
        f.write('\n')
    print(json.dumps(done), flush=True)


def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else ''
    if mode == 'preflight':
        assert len(sys.argv) == 2, 'usage: preflight'
        s = setup()
        _, provenance = runtime(s.h)
        print(json.dumps(dict(preflight='passed', bindings=len(s.bound), runtime=provenance, writes=0)), flush=True)
        return
    assert mode in ('inputs', 'extract') and len(sys.argv) == 3, 'usage: inputs|extract CHAPTER'
    c = sys.argv[2]
    assert c and all(x.isalnum() or x == '_' for x in c), c
    job, inputs = JOBS/('csd-apportionment-pictures678-'+c), Q/('csd-apportionment-pictures678-'+c+'-inputs.json')
    s = setup()
    if mode == 'extract':
        extract(s, c, job, inputs)
        return
    assert not job.exists() and not inputs.exists(), ('never overwrite or retry', c)
    plan, _, files = build(s, c)
    with inputs.open('x') as f:
        json.dump(dict(files=[dict(path=str(p), sha256=s.h.sha(p)) for p in files]), f, indent=2)
        f.write('\n')
    print(json.dumps(dict(inputs=str(inputs), sha256=s.h.sha(inputs), bindings=len(files), merge=plan['merge'],
                          launched=False)), flush=True)


if __name__ == '__main__':
    main()
