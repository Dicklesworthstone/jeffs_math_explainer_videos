"""Read-only audio seam measurement of the complex master993 at its 14 chapter joins. Draft for root review.

Not launched. Root reviews it, places it in Q on CSD and serializes it after the 995 lane (verify995 and both
picture jobs terminal). It renders, synthesizes and regenerates nothing and changes no input.

Raw measurements, with no acceptance threshold:
  - The master's AAC is decoded once by the pinned ffmpeg to 32-bit float PCM and streamed. The sha256 and frame count
    of the whole decoded stream are recorded.
  - Around each of the 14 boundary samples (cumulative chapter frames x 800, re-derived from the master993 summary),
    per channel: the samples at the seam, the first and second differences across it, how many samples in the window
    jump further, the largest jumps in the window and within 10 ms, peaks, RMS before and after, 10 ms block RMS,
    exact-zero counts and the exact-zero run nearest the seam.
  - The same for the kit's post-gain float master WAV, which is the exact AAC encoder input (explainer_kit cli.rs
    master_video: replay, concatenate, apply_master_gain, wav::write_master F32, then master::join encodes it).
    This is done only when its provenance and layout are proven:
      - it is in master993's own fresh out dir, with an mtime inside that job's window and the pinned sha256;
      - master993's frozen inputs bind the kit sources, binary 268 and ffmpeg at the hashes pinned here;
      - its header is WAVE_FORMAT_EXTENSIBLE IEEE float, 2 channels, 48 kHz, with fact and data both 106929600 frames.
  - When that proof holds, the residual (decoded minus WAV) in each window, at lags 0, +-1, +-1024 and +-2112, so the
    alignment is shown rather than assumed.
  - Every window is saved as raw interleaved float32 little-endian, decoded and WAV, with sha256.

Not claimed: listening, ASR or word checks, pacing, fine sync, AV or release acceptance, or any codec-error limit.

usage (on CSD, /usr/bin/python3 -I -B):
  master993-audio-seams01.py check      read-only gates and lane status; writes nothing
  master993-audio-seams01.py launch     gates, then writes the inputs file (x) and runs the guard (native) -> measure
  master993-audio-seams01.py measure --job JOB --inputs INPUTS     only under the guard
  master993-audio-seams01.py selftest   synthetic arrays only; reads and writes no file
"""
import array
import hashlib
import heapq
import json
import math
import shutil
import subprocess
import sys
from datetime import datetime
from pathlib import Path

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
GUARD = D/'guard_csd02.py'
PYTHON = '/usr/bin/python3'
FFMPEG, FFPROBE = Path('/usr/bin/ffmpeg'), Path('/usr/bin/ffprobe')
JOB = 'csd-master993-audio-seams01'
INPUTS = Q/'master993-audio-seams01-inputs.json'
MJOB = D/'jobs/csd-combined-master993'
MOVIE = MJOB/'final/complex_analysis_visual_master.mp4'
WAV = MJOB/'final/master/complex_analysis_visual.master.wav'
SUMMARY = MJOB/'master-summary.json'
MLOG = MJOB/'master-native.log'
MRECEIPT = MJOB/'resource-receipt.json'
MINPUTS = Q/'combined-master993-inputs.json'
KIT = D/'work/tree/repo-view/crates'
BIN268 = D/'work/bin/review-repair268'
PINS = {
    MOVIE: '80244e88ca5970d0da07a5d96d6d825ae1851bae703d75bd200f8810b04e7b90',
    WAV: 'e4b1014f1e09314ab686bab077741e1b62838340c435a4940661006c1953ecc5',
    SUMMARY: '38b52105520640e495ada17aa00da4ddbc16316a5a77c4568e102e1ca3adf185',
    MLOG: '0d3a8d28f17a4881be5e896ec2ed08ca09276eebbf2aee529cfd1dccf2c22dd3',
    MINPUTS: 'a610afcdee1c948e397f250405546c636a9654a6cef94b31ce6ef8d3f541ca91',
    Q/'combined-master993.py': '1955ca3202c8a4f382fea1682ac754eadf19fd708b2ef983c3f91f12607b6edb',
    KIT/'explainer_kit/src/cli.rs': '3c5221febc3e3d92d37aa97e2f8d38c319121215a67a1dc68d24534d41d862c9',
    KIT/'explainer_kit/src/master.rs': '7d1952dbea0be5220aee9c768cd57937f99c168de23b89751c9d94da5f40270d',
    KIT/'explainer_score/src/wav.rs': '0f41938b7d1c8f43267066ac205e9d4097d210cb26fb448a643d675508d1b729',
    BIN268: '983954c508e42b90d443ad6ca09f2baba7565a4f609d09d2ea821640534d6ec7',
    FFMPEG: 'bdf6aabffdba7411edff8d36c389d695257fcdf823d196020176e117612862f6',
    FFPROBE: 'a6dac1e9e8631e04075d06854cc4069308aec279bb04683b4a0bca5e4f54b2fe',
}
# Files master993's frozen inputs must bind at the same hashes (the job that wrote the WAV).
BOUND_BY_MASTER = (KIT/'explainer_kit/src/cli.rs', KIT/'explainer_kit/src/master.rs', KIT/'explainer_score/src/wav.rs',
                   BIN268, FFMPEG, FFPROBE, Q/'combined-master993.py')
VERIFY = D/'jobs/csd-final-verify995'
PICTURES = str(D/'jobs/csd-final-pictures995-%s')
PICTURE_PARTS = ('overview', 'native')
LAUNCH_FREE_BYTES = 12*1024**3

RATE, SPF = 48000, 800                     # 48 kHz, 800 samples per 60 fps frame
TOTAL_FRAMES = 133662
TOTAL = TOTAL_FRAMES*SPF                   # 106929600 sample frames
BOUNDARIES = (5048800, 9346400, 15145600, 20564800, 28468000, 36492000, 41684800,
              53652000, 66855200, 72808800, 82354400, 85508800, 94021600, 103426400)
W = 2*RATE                                 # window: 2 s each side of the seam
NEAR = RATE//100                           # 10 ms each side
BLOCK = RATE//100                          # 10 ms RMS blocks
LAGS = (-2112, -1024, -1, 0, 1, 1024, 2112)
PAD = max(abs(x) for x in LAGS)
HEADER = 80                                # wav.rs write_master F32: RIFF 12 + fmt 48 + fact 12 + data 8
FLOAT_GUID = bytes([0x03, 0, 0, 0, 0, 0, 0x10, 0, 0x80, 0, 0, 0xAA, 0, 0x38, 0x9B, 0x71])
NOT_CLAIMED = dict(listening=False, asr_or_words=False, pacing=False, fine_sync=False, av_acceptance=False,
                   release_acceptance=False, codec_error_limit=None, acceptance_thresholds=None)


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8'))


def success(path):
    r = read(path)
    assert r.get('admitted') and r.get('finished') and r.get('exit') == 0, path
    assert r['inputs_sha256'] == r['final_inputs_sha256'], path
    assert not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), path
    return r


def when(text):
    return datetime.fromisoformat(text).timestamp()


def db(x):
    return None if x == 0 else round(20*math.log10(x), 3)


# ---------- gates ----------

def boundaries_from_summary():
    s = read(SUMMARY)
    assert s['frames'] == TOTAL_FRAMES and s['outputs']['master']['sha256'] == PINS[MOVIE]
    assert s['outputs']['master']['path'] == str(MOVIE)
    at, out = 0, []
    for row in s['rows'][:-1]:
        at += row['frames']
        out.append(at*SPF)
    assert sum(r['frames'] for r in s['rows']) == TOTAL_FRAMES and tuple(out) == BOUNDARIES
    return [r['chapter'] for r in s['rows']]


def master_log(chapters):
    lines = MLOG.read_text(encoding='utf-8').splitlines()
    replays = [x for x in lines if "replay = fmn's soundtrack, bit for bit" in x]
    assert [x.split()[0] for x in replays] == chapters, 'replay lines are not the 15 chapters'
    masters = [x for x in lines if x.startswith('master -> ')]
    assert len(masters) == 1 and str(MOVIE) in masters[0]
    return dict(replay_lines=replays, master_line=masters[0])


def wav_layout():
    """The header wav.rs write_master writes for Format::F32, read field by field. Returns (fields, problems)."""
    with WAV.open('rb') as stream:
        h = stream.read(HEADER)
    u = lambda a, b: int.from_bytes(h[a:b], 'little')
    f = dict(riff=h[0:4].decode('latin-1'), riff_size=u(4, 8), wave=h[8:12].decode('latin-1'),
             fmt=h[12:16].decode('latin-1'), fmt_size=u(16, 20), tag=u(20, 22), channels=u(22, 24), rate=u(24, 28),
             byte_rate=u(28, 32), block_align=u(32, 34), bits=u(34, 36), cb_size=u(36, 38), valid_bits=u(38, 40),
             channel_mask=u(40, 44), float_guid=h[44:60] == FLOAT_GUID, fact=h[60:64].decode('latin-1'),
             fact_size=u(64, 68), fact_frames=u(68, 72), data=h[72:76].decode('latin-1'), data_bytes=u(76, 80),
             file_bytes=WAV.stat().st_size)
    want = dict(riff='RIFF', riff_size=72 + TOTAL*8, wave='WAVE', fmt='fmt ', fmt_size=40, tag=0xFFFE, channels=2,
                rate=RATE, byte_rate=RATE*8, block_align=8, bits=32, cb_size=22, valid_bits=32, channel_mask=3,
                float_guid=True, fact='fact', fact_size=4, fact_frames=TOTAL, data='data', data_bytes=TOTAL*8,
                file_bytes=HEADER + TOTAL*8)
    return f, ['%s %r, expected %r' % (k, f[k], v) for k, v in want.items() if f[k] != v]


def provenance():
    """Why the WAV is the AAC encoder input of this master. Returns (record, problems)."""
    problems = []
    receipt = success(MRECEIPT)
    mtime = WAV.stat().st_mtime
    if not when(receipt['started']) < mtime < when(receipt['finished']):
        problems.append('WAV mtime outside the master993 job window')
    if sha(MINPUTS) != receipt['inputs_sha256']:
        problems.append('combined-master993-inputs.json is not the inputs the master job ran with')
    bound = {row['path']: row['sha256'] for row in read(MINPUTS)['files']}
    for path in BOUND_BY_MASTER:
        if bound.get(str(path)) != PINS[path]:
            problems.append('master993 inputs do not bind %s at %s' % (path, PINS[path][:8]))
    if not str(WAV).startswith(str(MJOB/'final') + '/'):
        problems.append('WAV is not in the master job out dir')
    return dict(master_job_started=receipt['started'], master_job_finished=receipt['finished'],
                wav_mtime=datetime.fromtimestamp(mtime).astimezone().isoformat(),
                master_inputs_sha256=receipt['inputs_sha256'], writer='explainer_kit cli.rs master_video: '
                'apply_master_gain then wav::write_master(F32), then master::join encodes it as AAC 256k'), problems


def lane_status():
    v = VERIFY/'resource-receipt.json'
    status = dict(verify995=read(v) if v.exists() else None)
    for part in PICTURE_PARTS:
        p = Path(PICTURES % part)/'resource-receipt.json'
        status['pictures995-' + part] = read(p) if p.exists() else None
    brief = {k: None if r is None else dict(finished=r.get('finished'), exit=r.get('exit')) for k, r in status.items()}
    terminal = brief['verify995'] is not None and bool(brief['verify995']['finished'])
    if terminal and brief['verify995']['exit'] == 0:
        terminal = all(brief['pictures995-' + p] and brief['pictures995-' + p]['finished'] for p in PICTURE_PARTS)
    return brief, terminal


def gates():
    for path, digest in PINS.items():
        assert sha(path) == digest, str(path)
    chapters = boundaries_from_summary()
    log = master_log(chapters)
    layout, bad_layout = wav_layout()
    prov, bad_prov = provenance()
    return dict(chapters=chapters, master_log=log, wav_layout=layout, wav_layout_problems=bad_layout,
                provenance=prov, provenance_problems=bad_prov, compare=not bad_layout and not bad_prov)


# ---------- measurement ----------

def deinterleave(a):
    return [list(a[0::2]), list(a[1::2])]


def zero_runs(x):
    runs, start = [], None
    for i, v in enumerate(x):
        if v == 0.0 and start is None:
            start = i
        elif v != 0.0 and start is not None:
            runs.append((start, i))
            start = None
    if start is not None:
        runs.append((start, len(x)))
    return runs


def channel_stats(x, seam, near=NEAR, block=BLOCK):
    """x: one channel's window; seam: index of the first sample after the boundary. Offsets are relative to it."""
    d1 = [0.0] + [abs(x[i] - x[i-1]) for i in range(1, len(x))]          # d1[i]: jump into sample i
    d2 = [0.0] + [abs(x[i+1] - 2*x[i] + x[i-1]) for i in range(1, len(x)-1)] + [0.0]   # centred at i
    jump = x[seam] - x[seam-1]
    sec = (x[seam] - 2*x[seam-1] + x[seam-2], x[seam+1] - 2*x[seam] + x[seam-1])
    lo, hi = seam - near, seam + near
    top = heapq.nlargest(5, range(1, len(x)), key=d1.__getitem__)
    nj = max(range(max(1, lo), hi), key=d1.__getitem__)
    peak = max(range(len(x)), key=lambda i: abs(x[i]))
    npk = max(range(lo, hi), key=lambda i: abs(x[i]))
    rms = lambda v: math.sqrt(sum(t*t for t in v)/len(v))
    runs = zero_runs(x)
    nearest = min(runs, key=lambda r: 0 if r[0] <= seam < r[1] else min(abs(r[0]-seam), abs(r[1]-1-seam)), default=None)
    longest = max(runs, key=lambda r: r[1]-r[0], default=None)
    rel = lambda r: None if r is None else dict(start=r[0]-seam, end=r[1]-seam, length=r[1]-r[0])
    return dict(
        samples_at_seam=dict(zip(('-2', '-1', '0', '+1'), x[seam-2:seam+2])),
        jump=jump, second_differences=dict(at_minus1=sec[0], at_0=sec[1]),
        larger_jumps_in_window=sum(1 for v in d1 if v > abs(jump)),
        larger_second_differences_in_window=sum(1 for v in d2 if v > max(abs(sec[0]), abs(sec[1]))),
        top_jumps=[dict(offset=i-seam, abs_jump=d1[i]) for i in top],
        largest_jump_within_10ms=dict(offset=nj-seam, abs_jump=d1[nj]),
        peak=dict(offset=peak-seam, abs=abs(x[peak]), dbfs=db(abs(x[peak]))),
        peak_within_10ms=dict(offset=npk-seam, abs=abs(x[npk]), dbfs=db(abs(x[npk]))),
        rms_dbfs_before=db(rms(x[:seam])), rms_dbfs_after=db(rms(x[seam:])),
        block_rms_dbfs=dict(block_samples=block, first_block_offset=-seam,
                            values=[db(rms(x[k:k+block])) for k in range(0, len(x) - block + 1, block)]),
        exact_zeros=dict(count=sum(1 for v in x if v == 0.0), runs=len(runs), nearest_run=rel(nearest),
                         longest_run=rel(longest)))


def residuals(dec, ref, seam, pad=PAD, lags=LAGS, near=NEAR):
    """dec covers [b-W, b+W); ref covers [b-W-pad, b+W+pad). A lag L compares dec[i] with ref at i-L."""
    out = {}
    n = len(dec)
    for lag in lags:
        r = [dec[i] - ref[i - lag + pad] for i in range(n)]
        row = dict(rms_dbfs=db(math.sqrt(sum(t*t for t in r)/n)))
        if lag == 0:
            k = max(range(n), key=lambda i: abs(r[i]))
            w = r[seam-near:seam+near]
            row.update(largest=dict(offset=k-seam, abs=abs(r[k]), dbfs=db(abs(r[k]))),
                       rms_dbfs_within_10ms=db(math.sqrt(sum(t*t for t in w)/len(w))),
                       at_seam=dict(zip(('-1', '0'), r[seam-1:seam+1])))
        out[str(lag)] = row
    out['lowest_rms_lag'] = min(lags, key=lambda L: -1e9 if out[str(L)]['rms_dbfs'] is None else out[str(L)]['rms_dbfs'])
    return out


def decode(job):
    """One decode of the master's audio to f32le, streamed: the 14 windows kept, the whole stream hashed and counted."""
    keep = {b: bytearray() for b in BOUNDARIES}
    digest, pos = hashlib.sha256(), 0
    argv = [str(FFMPEG), '-nostdin', '-v', 'error', '-i', str(MOVIE), '-map', '0:a:0', '-vn', '-sn', '-dn',
            '-c:a', 'pcm_f32le', '-f', 'f32le', 'pipe:1']
    with (job/'decode.log').open('x') as log:
        proc = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=log)
        while chunk := proc.stdout.read(1 << 23):
            digest.update(chunk)
            for b in BOUNDARIES:
                lo, hi = max((b - W)*8, pos), min((b + W)*8, pos + len(chunk))
                if lo < hi:
                    keep[b] += chunk[lo - pos:hi - pos]
            pos += len(chunk)
        assert proc.wait(timeout=3600) == 0, 'ffmpeg decode failed'
    assert (job/'decode.log').stat().st_size == 0, 'ffmpeg wrote to stderr; see decode.log'
    assert pos % 8 == 0
    windows = {}
    for b, raw in keep.items():
        assert len(raw) == 2*W*8, (b, len(raw))
        windows[b] = array.array('f', bytes(raw))
    return windows, dict(argv=argv, decoded_bytes=pos, decoded_frames=pos//8, expected_frames=TOTAL,
                         decoded_frames_minus_expected=pos//8 - TOTAL, decoded_sha256=digest.hexdigest())


def wav_window(b):
    with WAV.open('rb') as stream:
        stream.seek(HEADER + (b - W - PAD)*8)
        raw = stream.read((2*W + 2*PAD)*8)
    assert len(raw) == (2*W + 2*PAD)*8
    return array.array('f', raw)


def save(job, name, a):
    path = job/'windows'/name
    with path.open('xb') as stream:
        a.tofile(stream)
    return dict(path=str(path), sha256=sha(path), layout='interleaved L,R float32 little-endian, 48 kHz')


def measure(args):
    assert sys.byteorder == 'little' and array.array('f').itemsize == 4
    assert args[:1] == ['--job'] and args[1] == JOB and args[2:3] == ['--inputs'] and Path(args[3]) == INPUTS
    job = D/'jobs'/JOB
    receipt = read(job/'resource-receipt.json')
    assert receipt['admitted'] and 'exit' not in receipt
    assert receipt['policy']['stop_free_gib'] == 8 and receipt['policy']['phase_growth_gib'] == 4
    assert sha(INPUTS) == receipt['inputs_sha256']
    bound = {row['path']: row['sha256'] for row in read(INPUTS)['files']}
    for path, digest in bound.items():
        assert sha(path) == digest, path
    for path in (Path(__file__).resolve(), *PINS):
        assert bound.get(str(path)) == sha(path), str(path)
    g = gates()
    (job/'windows').mkdir()
    probe = json.loads(subprocess.check_output([str(FFPROBE), '-v', 'error', '-select_streams', 'a:0', '-show_streams',
                                                '-show_format', '-of', 'json', str(MOVIE)]))
    a = probe['streams'][0]
    assert (a['codec_name'], a['sample_rate'], a['channels']) == ('aac', '48000', 2)
    dec, facts = decode(job)
    joins = []
    for n, b in enumerate(BOUNDARIES):
        left, right = g['chapters'][n], g['chapters'][n+1]
        d = deinterleave(dec[b])
        row = dict(index=n+1, left=left, right=right, boundary_sample=b, boundary_seconds=b/RATE,
                   window=dict(start_sample=b - W, end_sample=b + W),
                   decoded=dict(files=save(job, 'join%02d-decoded.f32' % (n+1), dec[b]),
                                channels={c: channel_stats(d[k], W) for k, c in enumerate(('L', 'R'))}))
        if g['compare']:
            ref = wav_window(b)
            r = deinterleave(ref)
            row['wav'] = dict(files=dict(save(job, 'join%02d-wav.f32' % (n+1), ref), start_sample=b - W - PAD),
                              channels={c: channel_stats(r[k][PAD:PAD + 2*W], W) for k, c in enumerate(('L', 'R'))})
            row['decoded_minus_wav'] = {c: residuals(d[k], r[k], W) for k, c in enumerate(('L', 'R'))}
        joins.append(row)
    for path, digest in bound.items():
        assert sha(path) == digest, path
    result = dict(helper_sha256=sha(Path(__file__).resolve()), inputs_sha256=sha(INPUTS), master=str(MOVIE),
                  master_sha256=PINS[MOVIE], wav=str(WAV), wav_sha256=PINS[WAV], ffprobe_audio=probe, decode=facts,
                  gates={k: v for k, v in g.items() if k != 'master_log'}, master_log=g['master_log'],
                  window_samples_each_side=W, near_samples=NEAR, lags=list(LAGS), joins=joins, not_claimed=NOT_CLAIMED,
                  scope='Raw seam statistics of the encoded master993 audio (and its post-gain float WAV when proven). '
                        'No thresholds, no listening, no ASR, no pacing or AV acceptance.')
    with (job/'seams.json').open('x') as stream:
        json.dump(result, stream, indent=1)
    return 0


def check():
    g = gates()
    lane, terminal = lane_status()
    print(json.dumps(dict({k: v for k, v in g.items() if k != 'master_log'}, master_line=g['master_log']['master_line'],
                          lane=lane, lane_terminal=terminal), indent=1))
    return 0


def launch():
    g = gates()
    lane, terminal = lane_status()
    assert terminal, 'the 995 lane is not terminal: %r' % (lane,)
    job = D/'jobs'/JOB
    assert not job.exists() and not INPUTS.exists()
    assert shutil.disk_usage(D/'jobs').free >= LAUNCH_FREE_BYTES, 'free space below 12 GiB; nothing written'
    files = sorted({Path(__file__).resolve(), MRECEIPT, Path(PYTHON).resolve(), *PINS}, key=str)
    with INPUTS.open('x') as stream:
        json.dump({'files': [dict(path=str(p), sha256=sha(p)) for p in files]}, stream, indent=2)
    print('compare with WAV:', g['compare'])
    subprocess.run([PYTHON, '-I', '-B', str(GUARD), '--phase', 'native', '--job', JOB, '--inputs', str(INPUTS),
                    '--', PYTHON, '-I', '-B', str(Path(__file__).resolve()), 'measure', '--job', JOB,
                    '--inputs', str(INPUTS)], check=True)
    success(job/'resource-receipt.json')
    return 0


def selftest():
    """Synthetic only: a sine with a step at the seam, an exact-zero run, and a copy delayed by 1024 samples."""
    w, near = 4800, 48
    x = [0.01*math.sin(2*math.pi*440*i/RATE) for i in range(2*w)]       # zero-run edges jump at most 0.01
    x[w:] = [v + 0.1 for v in x[w:]]
    x[w-300:w-100] = [0.0]*200
    s = channel_stats(x, w, near=near, block=480)
    assert abs(s['jump'] - (x[w] - x[w-1])) < 1e-12 and s['larger_jumps_in_window'] == 0, s['larger_jumps_in_window']
    assert s['top_jumps'][0]['offset'] == 0 and s['largest_jump_within_10ms']['offset'] == 0
    assert s['exact_zeros']['longest_run'] == dict(start=-300, end=-100, length=200), s['exact_zeros']
    assert s['exact_zeros']['nearest_run'] == s['exact_zeros']['longest_run']
    smooth = channel_stats([0.01*math.sin(2*math.pi*440*i/RATE) for i in range(2*w)], w, near=near, block=480)
    assert smooth['larger_jumps_in_window'] > 0
    pad = 2112
    ref = [0.3*math.sin(2*math.pi*(97 + i % 5)*i/RATE) for i in range(2*w + 2*pad)]
    dec = [ref[i - 1024 + pad] for i in range(2*w)]
    r = residuals(dec, ref, w, pad=pad, lags=(-1024, 0, 1024), near=near)
    assert r['lowest_rms_lag'] == 1024 and r['1024']['rms_dbfs'] is None, r
    assert zero_runs([0.0, 1.0, 0.0, 0.0]) == [(0, 1), (2, 4)]
    print('selftest OK (synthetic arrays only)')
    return 0


if __name__ == '__main__':
    mode, rest = sys.argv[1], sys.argv[2:]
    if mode == 'selftest':
        sys.exit(selftest())
    assert Path(__file__).resolve().parent == Q, 'run from Q on CSD'
    sys.exit({'check': lambda: check(), 'launch': lambda: launch(), 'measure': lambda: measure(rest)}[mode]())
