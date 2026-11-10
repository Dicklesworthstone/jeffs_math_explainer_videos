"""Guarded technical verification of the CSD mixed-host apportionment master735. Prepared for root; never executed by its author.

Successor of verify701. On the CSD only, with /usr/bin/python3 -I -B:
  inputs   no job: requires a stable successful master735 at its reviewed hash, freezes its whole closure and every file it
           wrote.
  run      only as the child of guard_csd02.

What differs from verify701:
  1. It reads master735's job, inputs manifest, summary and materialized record. Both modes refuse until root sets
     MASTER_HELPER_SHA to master735's final reviewed hash, which changes when the real chapter 06 manifest is bound. The
     master's frozen manifest and its summary must carry that hash, and the summary's chapter 06 manifest hash must be the
     frozen one.
  2. The unchanged caption checks below now read final/'s repaired pair. Before any decode, caption_pairs also binds both
     of master735's pairs at record 734's hashes: the generated pair in captions-generated735/, and the repaired pair in
     captions-repaired735/, which final/ must hold as the same files. It re-runs the pinned helper 727 in memory on the
     generated pair and requires the repaired bytes exactly, and requires that the parse differs only in caption 168's
     end, 616290 to 616310 ms. That is a display dwell repair; nothing here is evidence of speech timing.

Every other check of verify701 keeps its original limits: per-cue placement from the exact 24 kHz mono recordings, script
order, captions (SRT = VTT, words exact, 42 characters, two lines, at most 20 characters per second), master streams, a full
-xerror audio and video decode with per-frame SHA-256, per-frame equality of the master against all 15 chapter movies in
order, and the loudness and true-peak limits. No limit changed for the host. The chapters encoded on the Mac are decoded
here by the same ffmpeg as those encoded here, and the frame comparison is the actual test that the join kept every picture.

It reports no picture, motion, audio, fine-sync, independent AV or release result; those stay open and are recorded false.
"""
import hashlib
import importlib.util
import json
import os
import re
import signal
import subprocess
import sys
import time
import wave
from pathlib import Path

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
JOBS = D/'jobs'
A = JOBS/'csd-apportionment-assembly664'
MASTER = JOBS/'csd-apportionment-master770'
MASTER_INPUTS = Q/'root-apportionment770-inputs.json'
MASTER_HELPER = Q/'root-apportionment770-master-bound.py'
# Root installs master735's final reviewed sha256 here once the real chapter 06 manifest is bound; until then both modes refuse.
MASTER_HELPER_SHA = '220f9cf18bc146cea4333f3f53886436cccdc4450b27e72772ccd733213227b3'
JOB = JOBS/'csd-apportionment-verify771'
INPUTS = Q/'root-apportionment771-inputs.json'
HELPER = Path(__file__).resolve()
CONTROLLER = Q/'controller702.py'
SCRIPT = A/'repo/explainer_corpus/apportionment/script.tsv'
SUPPORT = Q/'apportionment-csd-support671.py'
SUPPORT_SHA = '244fe1a47d5a40f71594031bafc09fefb45558c38ed72adbfb6aa1b1f269c40a'
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4, work_cap_gib=56, target_growth_cap_gib=20,
              sample_seconds=10)
PYTHON = '/usr/bin/python3'
FFMPEG, FFPROBE = '/usr/bin/ffmpeg', '/usr/bin/ffprobe'
TOTAL_FRAMES = 100886
CUES = 93
SCOPE = ('Technical checks only on the mixed-host master735: chapters 01-02 encoded on the Mac, 03-15 on the CSD, chapter 06 by '
         'its reviewed successor job. The caption 168 repair is checked as bytes and parse only: display dwell, not evidence '
         'of speech timing. No picture, motion, audio, fine-sync, independent AV or release acceptance.')
CAPTION_HELPER = Q/'apportionment-caption-repair727'/'repair_caption168.py'
CAPTION_HELPER_SHA = 'd94aab217a6ee6639328414b774a7a374fb610c0716f3213222ca421a3762b9f'
CAPTION_NAMES = ('apportionment.en.srt', 'apportionment.en.vtt')
# Root's record 734 (root-caption-actual734.json, sha256 3659f8fa12f535e5adbbe37d69f23110e27f9df6aeff66ec7a8bd395aee83324):
# helper 727 on master700's actual pair, before and after; master735 pins the same four hashes.
GENERATED_CAPTIONS = ('0568f2dd0f59cb2a207ba94426bb712f48a693a6bf2705d0dfb4c47a2b4aee9b',
                      '482c5817d52926f937e805cc3411904833a00878010559d216b507af85143690')
REPAIRED_CAPTIONS = ('61e636aea49ac7b8c0ab97f4a201a11932aa41e5c71d0bf66974bd7c236d96df',
                     '214e15cb76ad795adbd067ea89da52a3a344881d872bb058e7d8950268b5ef28')
CAPTION_FOLDERS = (('captions-generated735', GENERATED_CAPTIONS), ('captions-repaired735', REPAIRED_CAPTIONS))


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


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


def libraries(path):
    """The shared libraries a binary loads, as ldd resolves them; an unresolved one refuses."""
    out = subprocess.check_output(['/usr/bin/ldd', str(path)], text=True, timeout=60, env=dict(PATH='/usr/bin:/bin'))
    assert 'not found' not in out, path
    return {Path(m[1]).resolve() for m in (re.search(r'(/\S+)', line) for line in out.splitlines()) if m}


def load():
    assert sha(SUPPORT) == SUPPORT_SHA, 'support671 changed'
    spec = importlib.util.spec_from_file_location('owned671', SUPPORT)
    o = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(o)
    o.R, o.FFMPEG, o.FFPROBE = D, FFMPEG, FFPROBE
    return o


def stamp(s):
    assert re.fullmatch(r'\d{2,}:\d{2}:\d{2}[.,]\d{3}', s), s
    h, m, t = s.replace(',', '.').split(':')
    sec, ms = t.split('.')
    assert int(m) < 60 and int(sec) < 60
    return ((int(h)*60+int(m))*60+int(sec))*1000+int(ms)


def captions(p, frames, vtt=False):
    blocks = Path(p).read_text().strip().split('\n\n')
    out = []
    if vtt:
        assert blocks.pop(0) == 'WEBVTT'
    for i, block in enumerate(blocks, 1):
        lines = block.splitlines()
        if '-->' not in lines[0]:
            assert lines.pop(0) == str(i)
        a, z = map(stamp, lines.pop(0).split(' --> '))
        assert 0 <= a < z <= round(frames*1000/60)
        assert 1 <= len(lines) <= 2 and all(0 < len(x) <= 42 for x in lines)
        assert not out or a >= out[-1]['end_ms']
        out.append(dict(start_ms=a, end_ms=z, text='\n'.join(lines)))
    assert out
    return out


def frame_digests(p):
    result = []
    for line in Path(p).read_text().splitlines():
        if not line or line.startswith('#'):
            continue
        row = [x.strip() for x in line.split(',')]
        assert len(row) == 6 and row[0] == '0' and int(row[3]) == 1 and int(row[4]) > 0, row
        assert re.fullmatch('[0-9a-f]{64}', row[5]), row
        result.append((int(row[4]), row[5]))
    assert result
    return result


def require_master():
    assert MASTER_HELPER_SHA is not None, 'master735 is not bound: root installs its final reviewed sha256 in MASTER_HELPER_SHA'


def caption_helper():
    """Helper 727 by its pin, loaded the way support671 is."""
    assert sha(CAPTION_HELPER) == CAPTION_HELPER_SHA, 'caption helper 727 changed'
    spec = importlib.util.spec_from_file_location('repair727', CAPTION_HELPER)
    h = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(h)
    values = (h.CUE, h.START_MS, h.OLD_END_MS, h.NEW_END_MS, h.NEXT_START_MS, h.LIMIT, h.SRT_NAME, h.VTT_NAME)
    assert values == (168, 612110, 616290, 616310, 616900, 20, *CAPTION_NAMES), ('caption helper 727 values changed', values)
    return h


def caption_pairs(summary, out, bound):
    """Master735's caption 168 repair, proved again from frozen bytes before any decode. Display dwell only."""
    h = caption_helper()
    record = summary['caption_repair']
    assert (record['helper'], record['helper_sha256']) == (str(CAPTION_HELPER), CAPTION_HELPER_SHA), record['helper']
    assert bound.get(str(CAPTION_HELPER)) == CAPTION_HELPER_SHA, 'helper 727 is not frozen at its pin'
    pairs = {}
    for (name, pins), key in zip(CAPTION_FOLDERS, ('generated', 'repaired')):
        folder = MASTER/name
        assert record[key] == dict(folder=str(folder), srt_sha256=pins[0], vtt_sha256=pins[1]), (key, record[key])
        assert sorted(p.name for p in folder.iterdir()) == sorted(CAPTION_NAMES), folder
        for file, digest in zip(CAPTION_NAMES, pins):
            path = folder/file
            assert path.is_file() and not path.is_symlink() and bound.get(str(path)) == digest == sha(path), path
        pairs[key] = [(folder/n).read_bytes() for n in CAPTION_NAMES]
    assert record['final_hard_links'] == [str(out/n) for n in CAPTION_NAMES]
    assert (summary['srt_sha256'], summary['vtt_sha256']) == REPAIRED_CAPTIONS
    for file in CAPTION_NAMES:
        assert os.path.samefile(out/file, MASTER/CAPTION_FOLDERS[1][0]/file), ('final/ is not the repaired file', file)
    assert list(h.repair(*pairs['generated'])[:2]) == pairs['repaired'], 'helper 727 does not reproduce the repaired pair'
    report = record['report']
    assert (report['input_srt_sha256'], report['input_vtt_sha256']) == GENERATED_CAPTIONS
    assert (report['output_srt_sha256'], report['output_vtt_sha256']) == REPAIRED_CAPTIONS
    assert (report['cue'], report['start_ms'], report['end_ms_before'], report['end_ms_after'], report['next_start_ms'],
            report['gap_after_ms'], report['limit']) == (168, 612110, 616290, 616310, 616900, 590, 20), report
    assert 'not evidence of speech timing' in report['scope'] and 'not evidence of speech timing' in record['scope']
    generated, repaired = (MASTER/name for name, _ in CAPTION_FOLDERS)
    before = captions(generated/CAPTION_NAMES[0], TOTAL_FRAMES)
    assert before == captions(generated/CAPTION_NAMES[1], TOTAL_FRAMES, True)
    after = captions(repaired/CAPTION_NAMES[0], TOTAL_FRAMES)
    changed = [i for i, (x, y) in enumerate(zip(before, after), 1) if x != y]
    assert len(before) == len(after) == report['cues'] and changed == [168], changed
    assert (before[167]['end_ms'], after[167]) == (616290, dict(before[167], end_ms=616310)), after[167]
    fast = [i+1 for i, c in enumerate(before) if len(c['text'].replace('\n', ' '))*1000/(c['end_ms']-c['start_ms']) > 20]
    assert fast == [168], ('the generated pair was expected to fail at caption 168 alone', fast)
    return dict(generated=record['generated'], repaired=record['repaired'], helper=str(CAPTION_HELPER),
                helper_sha256=CAPTION_HELPER_SHA, cues=len(after), changed_cues=changed, generated_fast_captions=fast,
                helper_reproduces_repaired_pair=True, final_is_repaired_pair=True, display_dwell_only=True,
                speech_timing_evidence=False)


def freeze():
    """Verify639's closure: the master's whole frozen manifest, every file master735 wrote, this helper and the tools."""
    assert not JOB.exists() and not INPUTS.exists(), 'no duplicate job or manifest'
    assert HELPER.parent == Q, HELPER
    require_master()
    o = load()
    receipt = MASTER/'resource-receipt.json'
    o.success(receipt)
    d = read(receipt)
    assert d['command'] == [PYTHON, '-I', '-B', str(MASTER_HELPER), 'run'] and d['policy'] == POLICY, d['command']
    first = sha(receipt)
    time.sleep(15)
    assert sha(receipt) == first, 'the master receipt changed while being read'
    rows = {}

    def add(path, digest=None):
        path = Path(path)
        if str(path) in rows:
            assert digest is None or rows[str(path)] == digest, path
            return
        actual = sha(path)
        assert digest is None or actual == digest, path
        rows[str(path)] = actual

    add(MASTER_INPUTS, d['inputs_sha256'])
    for row in read(MASTER_INPUTS)['files']:
        add(row['path'], row['sha256'])
    for file in sorted(MASTER.rglob('*')):
        assert not file.is_symlink(), file
        if file.is_file():
            add(file)
    add(MASTER_HELPER, MASTER_HELPER_SHA)
    add(CAPTION_HELPER, CAPTION_HELPER_SHA)
    for name, pins in CAPTION_FOLDERS:
        for file, digest in zip(CAPTION_NAMES, pins):
            add(MASTER/name/file, digest)
    for file, digest in zip(CAPTION_NAMES, REPAIRED_CAPTIONS):
        add(MASTER/'final'/file, digest)
    tools = [Path(FFMPEG).resolve(), Path(FFPROBE).resolve()]
    python = Path(sys.executable).resolve()
    assert python == Path(PYTHON).resolve(), 'run with '+PYTHON
    for path in {HELPER, SUPPORT, SCRIPT, D/'guard_csd02.py', python, *tools, *libraries(python), *libraries(tools[0]),
                 *libraries(tools[1])}:
        add(path)
    if CONTROLLER.exists():
        add(CONTROLLER)
    assert not JOB.exists() and not INPUTS.exists(), 'no duplicate job or manifest'
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=k, sha256=v) for k, v in sorted(rows.items())]), f, indent=2)
    print('INPUTS742', len(rows), 'rows', sha(INPUTS), flush=True)


def guarded():
    r = live(JOB/'resource-receipt.json')
    assert r.get('admitted') is True and r.get('phase') == 'native' and r.get('preflight_only') is False, 'receipt state'
    assert 'exit' not in r and 'finished' not in r and r['policy'] == POLICY, 'receipt policy'
    assert r['command'] == [PYTHON, '-I', '-B', str(HELPER), 'run'], r['command']
    assert r['inputs_sha256'] == sha(INPUTS), 'manifest changed'
    assert sorted(p.name for p in JOB.iterdir()) == ['command.log', 'resource-receipt.json'], 'job folder is not fresh'
    frozen = read(INPUTS)['files']
    bound = {x['path']: x['sha256'] for x in frozen}
    assert len(bound) == len(frozen)
    return bound


def run():
    assert HELPER.parent == Q, HELPER
    require_master()
    o = load()
    o.own_group()
    o.STATE['job'] = JOB
    for sig in o.STOP_SIGNALS:
        signal.signal(sig, o.on_stop)
    bound = guarded()
    for path, digest in bound.items():
        assert sha(path) == digest, path
    os.sched_setaffinity(0, {0, 1, 2, 3})
    os.environ.update(PATH='/usr/bin:/bin')
    o.success(MASTER/'resource-receipt.json')
    summary, material = read(MASTER/'master735-summary.json'), read(MASTER/'materialized735.json')
    assert summary['frames'] == material['frames'] == TOTAL_FRAMES
    chapters = summary['chapters']
    assert len(chapters) == len(set(chapters)) == 15 and set(chapters) == set(material['chapters'])
    assert summary['imported_chapters'] == chapters[:2] == [c for c in chapters if material['chapters'][c]['imported']]
    out = MASTER/'final'
    movie = out/'apportionment_master.mp4'
    assert str(movie) == summary['movie'] and sha(movie) == summary['movie_sha256']
    assert sha(out/'apportionment.en.srt') == summary['srt_sha256'] and sha(out/'apportionment.en.vtt') == summary['vtt_sha256']
    assert summary['helper_sha256'] == MASTER_HELPER_SHA == bound.get(str(MASTER_HELPER)), 'not the reviewed master735'
    assert bound.get(summary['chapter06_manifest']) == summary['chapter06_manifest_sha256'], 'chapter 06 manifest is not frozen'
    script = dict(line.split('\t', 1) for line in SCRIPT.read_text().splitlines() if line and not line.startswith('#'))
    assert len(script) == CUES
    placements, offset = [], 0
    for ch in chapters:
        lines = [line.split('\t') for line in (out/(ch+'.timeline.tsv')).read_text().splitlines() if line and not line.startswith('#')]
        meta = {r[0]: r[1] for r in lines if len(r) == 2}
        frames = int(meta['frames'])
        assert int(meta['fps']) == 60 and meta['chapter'] == ch
        assert frames == material['chapters'][ch]['frames']
        previous = 0
        for row in lines:
            if row[0] != 'line':
                continue
            _, cue, start, gain, source = row
            start = int(start)
            assert source in bound and sha(source) == bound[source]
            with wave.open(source) as w:
                assert (w.getnchannels(), w.getsampwidth(), w.getframerate()) == (1, 2, 24000)
                samples = w.getnframes()
            end = start/60+samples/24000
            assert start/60 >= previous-1e-6 and end <= frames/60+1e-6
            previous = end
            placements.append(dict(cue=cue, chapter=ch, frame=offset+start, samples=samples, gain_db=float(gain), source=source))
        offset += frames
    assert offset == TOTAL_FRAMES and [x['cue'] for x in placements] == list(script)
    caps = captions(out/'apportionment.en.srt', offset)
    assert caps == captions(out/'apportionment.en.vtt', offset, True)
    assert ' '.join(' '.join(c['text'].split()) for c in caps) == ' '.join(' '.join(t.split()) for t in script.values())
    fast = [dict(index=i+1, **c) for i, c in enumerate(caps) if len(c['text'].replace('\n', ' '))*1000/(c['end_ms']-c['start_ms']) > 20]
    assert not fast, ('fast captions require repair', fast)
    repair = caption_pairs(summary, out, bound)

    def step(name, args, limit=14400):
        code, tracking = o.run_owned(name, args, limit)
        assert code == 0, (name, code)
        print('VERIFIED_STEP', name, tracking, flush=True)

    probe = JOB/'probe.json'
    step('probe', [FFPROBE, '-v', 'error', '-show_streams', '-show_format', '-of', 'json', '-o', str(probe), str(movie)], 120)
    streams = read(probe)['streams']
    assert len(streams) == 2
    v = next(s for s in streams if s['codec_type'] == 'video')
    a = next(s for s in streams if s['codec_type'] == 'audio')
    assert (v['codec_name'], v['width'], v['height'], v['r_frame_rate'], int(v['nb_frames']), v['pix_fmt']) == ('h264', 3840, 2160, '60/1', offset, 'yuv420p')
    assert all(v[k] == 'bt709' for k in ('color_space', 'color_transfer', 'color_primaries'))
    assert a['codec_name'] == 'aac' and int(a['sample_rate']) == 48000 and a['channels'] == 2
    assert abs(float(v['duration'])-offset/60) < .002 and abs(float(a['duration'])-offset/60) < .03
    final_hash = JOB/'master-frames.sha256'
    step('full-av-decode-and-video-hash', [FFMPEG, '-nostdin', '-v', 'error', '-xerror', '-threads', '4', '-i', str(movie), '-map', '0:v:0',
                                           '-fps_mode', 'passthrough', '-f', 'framehash', '-hash', 'sha256', str(final_hash),
                                           '-map', '0:a:0', '-f', 'null', '-'])
    expected = []
    for ch in chapters:
        target = JOB/(ch+'-frames.sha256')
        step('chapter-identity-'+ch, [FFMPEG, '-nostdin', '-v', 'error', '-xerror', '-threads', '4', '-i', str(out/(ch+'.mp4')), '-map', '0:v:0',
                                      '-fps_mode', 'passthrough', '-f', 'framehash', '-hash', 'sha256', str(target)])
        digests = frame_digests(target)
        assert len(digests) == material['chapters'][ch]['frames']
        expected.extend(digests)
    actual = frame_digests(final_hash)
    assert len(actual) == offset and actual == expected, 'Chapter-to-master decoded video differs'
    step('encoded-loudness', [FFMPEG, '-nostdin', '-hide_banner', '-nostats', '-threads', '4', '-i', str(movie), '-map', '0:a:0',
                              '-af', 'ebur128=peak=true', '-f', 'null', '-'])
    log = (JOB/'command.log').read_text(errors='replace')
    match = list(re.finditer(r'Integrated loudness:\s*I:\s*(-?[\d.]+) LUFS.*?True peak:\s*Peak:\s*(-?[\d.]+) dBFS', log, re.S))
    assert match, 'No final ebur128 summary'
    loud, peak = map(float, match[-1].groups())
    assert peak <= -1.0 and -30 < loud <= -15
    nominal_exception = loud < -17
    if nominal_exception:
        assert -1.7 <= peak <= -1.0, ('Quieter programme without ceiling binding', loud, peak)
    for path, digest in bound.items():
        assert sha(path) == digest, path
    result = dict(frames=offset, seconds=offset/60, placements=placements, captions=len(caps), srt_vtt_equal=True, caption_words_exact=True,
                  fast_captions=fast, full_av_decode=True, all_decoded_video_frames_match_chapters=True, lufs=loud, true_peak_dbfs=peak,
                  nominal_loudness_exception=nominal_exception, movie_sha256=sha(movie), imported_chapters=chapters[:2],
                  hosts=summary['hosts'], inputs_sha256=sha(INPUTS), helper_sha256=sha(HELPER), pictures_viewed=False,
                  audio_heard=False, fine_sync=False, independent_av_acceptance=False, scope=SCOPE,
                  caption_repair=repair, master_helper_sha256=MASTER_HELPER_SHA,
                  chapter06_manifest_sha256=summary['chapter06_manifest_sha256'])
    with (JOB/'verification742.json').open('x') as f:
        json.dump(result, f, indent=2)
    print('TECHNICAL742_COMPLETE; actual picture, motion, audio, fine-sync and independent AV review remain', flush=True)


if __name__ == '__main__':
    assert len(sys.argv) == 2 and sys.argv[1] in ('inputs', 'run'), 'usage: apportionment-csd-verify742.py inputs|run'
    (freeze if sys.argv[1] == 'inputs' else run)()
