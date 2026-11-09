"""Recovery 528 (CC2): finish the apportionment 540p narrated review preview that preview458 left at chapter 09.

Successor of 516 (cc2-apportionment-preview-recovery516.py, 81f39578, sealed, kept unchanged and never run). The
change is in the stop path only.

Why (CC0 526 M1):
- fmn starts every ffmpeg (the -version probe, the encoder, the audio mux) in a new process group: asupersync
  c48399f process.rs:1147 sets set_pgroup(0), and 1154 sets POSIX_SPAWN_SETPGROUP. fmn-platform a399d62
  process.rs:1749 uses that path. Each ffmpeg therefore leads a group of its own.
- Nothing on the binary's path handles SIGTERM.
- So 516's group stop, and the guard's, killed the binary but not its ffmpeg. 458 had the same behaviour.
- Once the binary is dead, launchd adopts the ffmpeg. A ppid walk from the helper can find it only while the
  binary is alive.

What 528 adds. Targets are always only the helper's own descendants that sit outside its process group. Each is
identified by pid, pgid, start time and command, never by name. The helper's own group is signalled once per step,
with killpg.
- **During each render**, every POLL_SECONDS (1 s), one ps snapshot is walked by ppid from the helper's pid. Every
  descendant outside the helper's group is remembered by its identity. This list survives the binary's death, which
  is what the guard's own group SIGTERM causes.
- **end_group()** runs on a timeout, on an exception, or on SIGTERM, SIGINT or SIGHUP. It:
  1. ignores further stop signals;
  2. sends SIGSTOP to a still-running binary, so it cannot start another ffmpeg;
  3. takes a fresh snapshot and adds any new escaped descendants;
  4. sends SIGTERM to every target that the snapshot shows to be the same process (pid, pgid, start and command);
  5. sends SIGTERM to the helper's group, then SIGCONT to the binary (a stopped process acts on SIGTERM only once
     continued);
  6. waits up to TERM_GRACE (6 s, under the guard's 20 s);
  7. re-verifies each target's identity, then sends SIGKILL to those still alive;
  8. writes <chapter>.cleanup.json with the targets, each signal's sent and skipped pids (with the reason), and the
     tracking counts;
  9. sends SIGKILL to the helper's group, itself included, so the receipt shows -9.
  If a step from 2 to 7 raises, the error is recorded, and steps 8 and 9 still run.
- **If ps cannot start** (for example EAGAIN), that sweep is recorded as not run, unverified targets are never
  signalled, and only the helper's own group is ended.
- **A render that exits by itself:** its tracked escaped descendants get TERM_GRACE to end on their own (an encoder
  stops at end of input). Any that are still the same process are then sent SIGTERM and, 2 s later, SIGKILL. The
  helper then fails (exit 1), so the chapter's files remain but are never counted.
- **Limit.** Suppose an ffmpeg starts less than one poll before a stop that kills the binary first (the guard's group
  SIGTERM). The helper never sees it: it ends at end of input (an encoder) or runs to completion (a mux). The helper's
  own stops cannot miss it, because they stop the binary before the snapshot.

Everything else is 516's unchanged: the pins, the 01-08 reuse proof (52-file listing fc5100da), chapter(), the
guard binding, the shared 14,400 s render budget, the pressure check, the outputs and the summary. Only the names
are new: the jobs, the summary, and the label that is printed. Root's commands, from the repository root:
  PY -I -B Q/guard_owner80g_target20g.py --phase native --job apportionment-frederick-preview528-check \
     --inputs MANIFEST528 -- PY -I -B Q/cc2-apportionment-preview-recovery528.py --check-only
  PY -I -B Q/guard_owner80g_target20g.py --phase native --job apportionment-frederick-preview528 \
     --inputs MANIFEST528 -- PY -I -B Q/cc2-apportionment-preview-recovery528.py
MANIFEST528 is root's new frozen manifest: preview458's 3,820 rows, this helper, the 52 preview458 files and the
original bindings. The helper does not read it. It records the receipt's inputs hash, its own hash and the binary's.
Both modes run only as the guard's own command, never unguarded: the check-only mode null-decodes the eight 01-08
movies.

Every check fails closed. An assertion stops the helper, and nothing already written is touched.
  Before rendering:
    - the job is fresh: only the guard's two files, admitted, native, the 80g policy, and this file as the command;
    - the binary, preview458.py, the inputs file, and all 3,820 of its rows are at their hashes;
    - timeline444 and score445 succeeded;
    - preview458's receipt is the failed one (a8d25c31), with no summary;
    - its 52 files are exactly the proven listing (fc5100da);
    - the binary lists the 15 chapters and every score QA passed;
    - chapters 01-08 pass chapter();
    - the new folder does not exist.
  Before each new chapter:
    - this user runs at most half of kern.maxprocperuid processes;
    - render time is left.
  chapter() checks:
    - timeline, cues and layout are byte-equal to timeline444's;
    - the cue log is timeline444's plus exactly the chapter's score stem rows, each at its hash and start sample;
    - the movie has one H.264 High yuv420p 960x540 60/1 stream with one packet per timeline frame, tv/bt709, and
      one AAC 48 kHz stereo stream within one AAC frame of the picture;
    - the encoder tags and the x264 settings are those of 01-08;
    - the movie decodes with -xerror;
    - the captured soundtrack is 16-bit stereo 48 kHz, frames x 800 samples.
    - Each passed chapter prints its file hashes and tracking counts, so a later recovery can reuse it by hash.
  After rendering: preview458's listing is checked again. Then preview528-summary.json is written (mode x).
Not checked: pictures, motion, sound, loudness, sync. This accepts nothing and authorizes no release.
"""
import hashlib
import json
import os
import signal
import struct
import subprocess
import sys
import time
from pathlib import Path

R = Path(__file__).resolve().parents[7]
G = R/'narration_lab/third-wave-20261024/root/elevenlabs-apportionment-rebuild-20261104/frederick-generation89'
B = R/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
BINARY = B/'apportionment-frederick-corrections-build209/apportionment-native209'
BINARY_SHA = 'c77303562bc062943796a55462a84f61af19262de760c0230e8318bd873b2df6'
PREVIEW458_SHA = '7ab178d17b2e0279ccf5940d9327dc691610c676e855228ed5391ae5be1c073d'
INPUTS, INPUTS_SHA = G/'preview458-inputs.json', '5817cb642dbe308bf2c838f64784b62ea613bdcf4678061ee34f0a2457e26024'
J458 = B/'apportionment-frederick-preview458'
RECEIPT458_SHA = 'a8d25c31a921545220e6911d73fc870c827dede9311d4a3cf00cb59e0fe4d96d'
LISTING458_SHA = 'fc5100dab6c12d9fbb320be155872e556501512b0719f57a6513bb7fa1a30ed3'   # see listing()
T444, S445 = B/'apportionment-frederick-timeline444', B/'apportionment-frederick-score445'
REF, STEMS = T444/'timeline', S445/'score/stems'
JOB, CHECK_JOB = B/'apportionment-frederick-preview528', B/'apportionment-frederick-preview528-check'
OUT = JOB/'preview'
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4, work_cap_gib=80, target_growth_cap_gib=20,
              sample_seconds=10)                       # guard_owner80g_target20g.py, as preview458's receipt records it
RENDER_SECONDS = 14400                                 # preview458's one render call; the seven calls share it
TERM_GRACE = 6                                         # the guard waits 20 s after its SIGTERM, then SIGKILL
POLL_SECONDS = 1.0                                     # how often a render's escaped descendants are looked for
PS = ['/bin/ps', '-ww', '-A', '-o', 'pid=,ppid=,pgid=,lstart=,comm=']
STOP_SIGNALS = (signal.SIGTERM, signal.SIGINT, signal.SIGHUP)
DONE = ('01_one_more_seat', '02_shares', '03_largest_tails', '04_sixteenth', '05_dials', '06_not_every_time',
        '07_yardstick', '08_signposts')
X264 = ('x264 - core 165 r3222 b35605a - H.264/MPEG-4 AVC codec - Copyleft 2003-2025 - http://www.videolan.org/x264.html'
        ' - options: cabac=1 ref=10 deblock=1:1:1 analyse=0x3:0x113 me=hex subme=8 psy=1 psy_rd=0.40:0.00 mixed_ref=1'
        ' me_range=16 chroma_me=1 trellis=2 8x8dct=1 cqm=0 deadzone=21,11 fast_pskip=1 chroma_qp_offset=-2 threads=17'
        ' lookahead_threads=2 sliced_threads=0 nr=0 decimate=1 interlaced=0 bluray_compat=0 constrained_intra=0'
        ' bframes=5 b_pyramid=2 b_adapt=1 b_bias=0 direct=3 weightb=1 open_gop=0 weightp=2 keyint=250 keyint_min=25'
        ' scenecut=40 intra_refresh=0 rc_lookahead=50 rc=crf mbtree=1 crf=16.0 qcomp=0.60 qpmin=0 qpmax=69 qpstep=4'
        ' ip_ratio=1.40 aq=1:0.60')
ENCODERS = ('Lavc62.28.102 libx264', 'Lavf62.12.102')            # the stream and container tags of 01-08
FFMPEG, FFPROBE = '/opt/homebrew/bin/ffmpeg', '/opt/homebrew/bin/ffprobe'
RATE, FPS = 48000, 60
STATE = dict(job=None, step=None, child=None, targets={}, polls=0, missed=0)


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def say(**fields):
    print(json.dumps(fields), flush=True)


def own_group():
    """The helper leads the group the guard made with start_new_session, so the group is this job and nothing else."""
    pid = os.getpid()
    assert os.getpgid(0) == pid and os.getsid(0) == pid, 'run 528 as the guard command itself, not unguarded or wrapped'


def table():
    """{pid: (ppid, pgid, start, command)} from one ps snapshot (read only)."""
    out = subprocess.run(PS, capture_output=True, text=True, check=True, timeout=10,
                         env=dict(os.environ, LC_ALL='C')).stdout
    rows = {}
    for line in out.splitlines():
        f = line.split(None, 8)                        # pid ppid pgid, five start fields, command
        if len(f) == 9:
            rows[int(f[0])] = (int(f[1]), int(f[2]), ' '.join(f[3:8]), f[8])
    return rows


def escaped(rows):
    """Identities of this helper's descendants (a ppid walk from its pid) that sit outside its process group."""
    me, group, kids, out = os.getpid(), os.getpgid(0), {}, {}
    for pid, row in rows.items():
        kids.setdefault(row[0], []).append(pid)
    todo = list(kids.get(me, []))
    while todo:
        pid = todo.pop()
        todo += kids.get(pid, [])
        _, pgid, start, command = rows[pid]
        if pgid != group:
            out['%d %s' % (pid, start)] = dict(pid=pid, pgid=pgid, start=start, command=command)
    return out


def look():
    """A fresh snapshot, adding any new escaped descendants to the targets; None if ps could not run."""
    try:
        rows = table()
    except (OSError, subprocess.SubprocessError, ValueError):
        STATE['missed'] += 1
        return None
    STATE['targets'].update(escaped(rows))
    return rows


def same(rows, t):
    row = rows.get(t['pid'])
    return row is not None and row[1:] == (t['pgid'], t['start'], t['command'])


def hit(sig, log):
    """Send sig to each target that a fresh snapshot shows to be the same process; log what was sent and skipped."""
    rows = look()
    if rows is None:
        log.append(dict(signal=int(sig), sweep='not run: ps could not start; no target was signalled'))
        return
    sent, skipped = [], []
    for t in STATE['targets'].values():
        if not same(rows, t):
            skipped.append(dict(pid=t['pid'], why='gone' if t['pid'] not in rows else 'another process now'))
            continue
        try:
            os.kill(t['pid'], sig)
            sent.append(t['pid'])
        except ProcessLookupError:
            skipped.append(dict(pid=t['pid'], why='gone'))
        except OSError as error:
            skipped.append(dict(pid=t['pid'], why=repr(error)))
    log.append(dict(signal=int(sig), sent=sent, skipped=skipped))


def alive_targets():
    rows = look()
    return [] if rows is None else [t['pid'] for t in STATE['targets'].values() if same(rows, t)]


def poke(pid, sig):
    try:
        os.kill(pid, sig)
    except ProcessLookupError:
        pass


def sweep(child, group, log):
    running = child is not None and child.poll() is None
    if running:
        poke(child.pid, signal.SIGSTOP)                # the binary can start no other ffmpeg during the sweep
    hit(signal.SIGTERM, log)                           # escaped groups: verified identities only
    os.killpg(group, signal.SIGTERM)                   # the binary, ffprobe, the checks' ffmpeg: all but this helper
    if running:
        poke(child.pid, signal.SIGCONT)                # a stopped process acts on SIGTERM only once continued
    deadline = time.monotonic() + TERM_GRACE
    if child is not None:
        try:
            child.wait(timeout=TERM_GRACE)
        except subprocess.TimeoutExpired:
            pass
    while time.monotonic() < deadline and alive_targets():
        time.sleep(0.25)
    hit(signal.SIGKILL, log)                           # re-verified, so a reused pid is never signalled


def end_group(reason):
    """End the escaped targets and the whole owned group, this helper last. Never returns."""
    for sig in STOP_SIGNALS:
        signal.signal(sig, signal.SIG_IGN)            # the helper outlives its own SIGTERM to write the record
    group, child, log = os.getpgid(0), STATE['child'], []
    try:
        sweep(child, group, log)
    except BaseException as error:                     # the group is still ended below
        log.append(dict(error=repr(error)))
    record = dict(reason=reason, step=STATE['step'], group=group, child_exit=None if child is None else child.poll(),
                  targets=list(STATE['targets'].values()), sweep=log, polls=STATE['polls'], missed=STATE['missed'],
                  then='SIGKILL to the whole group, this helper included')
    if STATE['job'] is not None:
        try:
            with (STATE['job']/('%s.cleanup.json' % (STATE['step'] or 'helper'))).open('x') as f:
                json.dump(record, f, indent=2)
        except OSError:
            pass
    sys.stdout.flush()
    sys.stderr.flush()
    os.killpg(group, signal.SIGKILL)
    os._exit(137)                                      # not reached: SIGKILL has ended this process


def on_stop(signum, frame):
    end_group('signal %d' % signum)


def read_receipt(job):
    for _ in range(10):                                # the guard rewrites it every 10 s while we run
        try:
            return json.loads((job/'resource-receipt.json').read_text())
        except json.JSONDecodeError:
            time.sleep(0.5)
    raise SystemExit('528 refused: the guard receipt could not be read')


def guard_binding(job, check_only):
    assert job.parent == B and job.is_dir() and not job.is_symlink(), ('no guard job folder', job)
    present = sorted(p.name for p in job.iterdir())
    assert present == ['command.log', 'resource-receipt.json'], ('the job folder is not fresh', present)
    r = read_receipt(job)
    assert r.get('admitted') is True and 'exit' not in r and 'finished' not in r, 'receipt state'
    assert r.get('phase') == 'native' and r.get('preflight_only') is False and r.get('policy') == POLICY, 'receipt policy'
    me = Path(__file__).resolve()
    assert any((R/a).resolve() == me for a in r['command']) and ('--check-only' in r['command']) == check_only, \
        ('the guard command is not this mode of this helper', r['command'])
    return dict(job=job.name, command=r['command'], inputs_sha256=r.get('inputs_sha256'), admission=r.get('admission'))


def success(receipt):
    d = json.loads(receipt.read_text())
    assert d.get('finished') and d.get('admitted') is True and d.get('preflight_only') is False and d.get('exit') == 0, receipt
    assert not any(d.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), receipt
    assert d['inputs_sha256'] == d['final_inputs_sha256'], receipt


def listing(job):
    """sha256 of 'sha256  bytes  relative path' lines, one per file, sorted by path (task 516's form)."""
    rows = []
    for rel in sorted(str(p.relative_to(job)) for p in job.rglob('*') if not p.is_dir()):
        assert not (job/rel).is_symlink(), rel
        rows.append('%s  %d  %s\n' % (sha(job/rel), (job/rel).stat().st_size, rel))
    return hashlib.sha256(''.join(rows).encode()).hexdigest()


def pressure():
    limit = int(subprocess.run(['/usr/sbin/sysctl', '-n', 'kern.maxprocperuid'], capture_output=True, text=True,
                               check=True, timeout=60).stdout)
    mine = subprocess.run(['/bin/ps', '-U', str(os.getuid()), '-o', 'pid='], capture_output=True, text=True,
                          check=True, timeout=60).stdout.split()
    return len(mine), limit


def stems(c):
    """{cue log path: (sha256, start sample)} of the chapter's score stem: one WAV, or the parts of a long stem."""
    whole, parts = STEMS/(c+'.music.f32.wav'), STEMS/(c+'.music.f32.parts.tsv')
    if not parts.exists():
        return {str(whole): (sha(whole), 0)}
    lines = parts.read_text().splitlines()
    assert lines[0] == 'score-parts\t1' and not whole.exists(), parts
    out = {}
    for row in lines[1:]:
        start, _, _, digest, name = row.split('\t')
        assert sha(STEMS/name) == digest, name
        out[str(STEMS/name)] = (digest, int(start))
    return out


def wav(path):
    data = path.read_bytes()
    assert data[:4] == b'RIFF' and data[8:12] == b'WAVE', path
    i, fmt = 12, None
    while i + 8 <= len(data):
        cid, n = data[i:i+4], struct.unpack('<I', data[i+4:i+8])[0]
        if cid == b'fmt ':
            fmt = struct.unpack('<HHIIHH', data[i+8:i+24])            # tag, channels, rate, byte rate, align, bits
        elif cid == b'data':
            return fmt, n // fmt[4]
        i += 8 + n + (n & 1)
    raise AssertionError('no data chunk in %s' % path)


def chapter(folder, c):
    """Check one finished chapter in folder; return the sha256 of its files."""
    for ext in ('timeline.tsv', 'cues.tsv', 'layout.tsv'):
        assert (folder/(c+'.'+ext)).read_bytes() == (REF/(c+'.'+ext)).read_bytes(), (c, ext, 'differs from timeline444')
    head = dict(ln.split('\t')[:2] for ln in (folder/(c+'.timeline.tsv')).read_text().splitlines()
                if ln.startswith(('chapter\t', 'fps\t', 'frames\t')))
    assert head['chapter'] == c and int(head['fps']) == FPS, c
    frames = int(head['frames'])
    want = stems(c)
    log = (folder/(c+'.cuelog.tsv')).read_text().splitlines()
    rows = [ln.split('\t') for ln in log]
    assert [ln for ln, r in zip(log, rows) if r[0] not in want] == (REF/(c+'.cuelog.tsv')).read_text().splitlines(), \
        (c, 'cue log is not timeline444 plus the stem')
    stem_rows = [r for r in rows if r[0] in want]
    assert sorted(r[0] for r in stem_rows) == sorted(want), (c, 'stem rows')
    assert all((r[1], int(r[12])) == want[r[0]] for r in stem_rows), (c, 'stem hash or start sample')
    movie = folder/(c+'.mp4')
    got = json.loads(subprocess.run([FFPROBE, '-v', 'error', '-count_packets', '-show_streams', '-show_format', '-of', 'json',
                                     str(movie)], check=True, capture_output=True, text=True, timeout=600).stdout)
    assert [s['codec_type'] for s in got['streams']] == ['video', 'audio'], c
    v, a = got['streams']
    assert (v['codec_name'], v['profile'], v['pix_fmt'], v['width'], v['height'], v['avg_frame_rate'], v['r_frame_rate'],
            v.get('color_range'), v.get('color_space')) == ('h264', 'High', 'yuv420p', 960, 540, '60/1', '60/1', 'tv', 'bt709'), (c, v)
    assert int(v['nb_read_packets']) == frames, (c, v['nb_read_packets'], frames)
    assert (a['codec_name'], int(a['sample_rate']), a['channels']) == ('aac', RATE, 2), (c, a)
    assert abs(float(a['duration']) - float(v['duration'])) <= 1024/RATE, (c, a['duration'], v['duration'])
    assert (v.get('tags', {}).get('encoder'), got['format'].get('tags', {}).get('encoder')) == ENCODERS, c
    data = movie.read_bytes()
    k = data.find(b'x264 - core')
    assert k >= 0 and data[k:data.index(b'\x00', k)].decode('ascii') == X264, (c, 'x264 settings differ from 01-08')
    p = subprocess.run([FFMPEG, '-nostdin', '-v', 'error', '-xerror', '-threads', '2', '-i', str(movie), '-f', 'null', '-'],
                       capture_output=True, text=True, timeout=1800)
    assert p.returncode == 0 and not p.stderr.strip(), (c, 'decode', p.returncode, p.stderr[-300:])
    tracks = folder/'soundtrack'/c
    n = 0
    while (tracks/('soundtrack_%d.wav' % (n+1))).exists():
        n += 1
    track = tracks/('soundtrack_%d.wav' % n)                   # the last capture, as the kit's master reads it
    fmt, samples = wav(track)
    assert (fmt[0], fmt[1], fmt[2], fmt[5]) == (1, 2, RATE, 16) and samples == (2*frames*RATE + FPS)//(2*FPS), (c, fmt, samples)
    files = {c+'.'+e: folder/(c+'.'+e) for e in ('mp4', 'timeline.tsv', 'cues.tsv', 'cuelog.tsv', 'layout.tsv')}
    files[str(track.relative_to(folder))] = track
    return dict(folder=str(folder.relative_to(R)), frames=frames, files={k: sha(p) for k, p in files.items()})


def settle():
    """After a render exits by itself: its tracked escaped descendants get TERM_GRACE to end on their own; any still
    the same process are then sent SIGTERM and, 2 s later, SIGKILL (verified each time). Returns the sweep log."""
    deadline = time.monotonic() + TERM_GRACE
    while time.monotonic() < deadline and alive_targets():
        time.sleep(0.25)
    if not alive_targets():
        return []
    log = []
    hit(signal.SIGTERM, log)
    time.sleep(2)
    hit(signal.SIGKILL, log)
    return log


def run_owned(name, argv, seconds):
    """Run argv in the guard-owned group, tracking its escaped descendants every POLL_SECONDS. From launch until it
    is settled, any exception, the timeout included, ends the targets and the whole group (end_group)."""
    STATE.update(step=name, targets={}, polls=0, missed=0)
    try:
        STATE['child'] = child = subprocess.Popen(argv, cwd=R)
        deadline = time.monotonic() + seconds
        while True:
            try:
                code = child.wait(timeout=max(0.0, min(POLL_SECONDS, deadline - time.monotonic())))
                break
            except subprocess.TimeoutExpired:
                if time.monotonic() >= deadline:
                    raise
            STATE['polls'] += 1
            look()
        leftovers = settle()
    except BaseException as error:
        end_group(repr(error))
    tracked = dict(polls=STATE['polls'], missed=STATE['missed'], escaped_seen=len(STATE['targets']))
    STATE['child'] = STATE['step'] = None
    assert not leftovers, (name, 'escaped descendants outlived the render; ended by identity', leftovers, tracked)
    return code, tracked


def render(c, seconds):
    cmd = [str(BINARY), 'render', c, '--res', '960x540', '--fps', '60', '--lint', '--narration', str(G/'narration437'),
           '--score', str(STEMS), '--out', str(OUT), '--crf', '16', '--preset', 'slow', '--threads', '4']
    print('RECOVERY528 COMMAND', cmd, 'timeout', round(seconds), flush=True)
    return run_owned(c, cmd, seconds)


def render_all(todo, record):
    left = RENDER_SECONDS
    for c in todo:
        done = [k for k in todo if k in record]
        mine, limit = pressure()
        say(chapter=c, processes=mine, limit=limit, render_seconds_left=round(left))
        if mine > limit // 2:
            raise SystemExit('process pressure: %d of %d processes; stopped before %s; finished %s' % (mine, limit, c, done))
        if left <= 0:
            raise SystemExit('render budget of %d s spent; stopped before %s; finished %s' % (RENDER_SECONDS, c, done))
        started = time.monotonic()
        code, tracked = render(c, left)
        left -= time.monotonic() - started
        assert code == 0, (c, 'render exit', code)
        record[c] = dict(chapter(OUT, c), tracker=tracked)
        say(chapter=c, passed=record[c])


def main(argv):
    assert argv in ([], ['--check-only']), 'usage: cc2-apportionment-preview-recovery528.py [--check-only]'
    check_only = argv == ['--check-only']
    own_group()
    for sig in STOP_SIGNALS:
        signal.signal(sig, on_stop)
    assert sys.version_info >= (3, 11), 'Python 3.11 or later'
    job = CHECK_JOB if check_only else JOB
    guard = guard_binding(job, check_only)
    STATE['job'] = job
    hashes = dict(helper_sha256=sha(__file__), binary_sha256=sha(BINARY), inputs_sha256=guard['inputs_sha256'])
    say(recovery528=job.name, **hashes)
    assert hashes['binary_sha256'] == BINARY_SHA and sha(G/'preview458.py') == PREVIEW458_SHA and sha(INPUTS) == INPUTS_SHA
    for receipt in (T444, S445):
        success(receipt/'resource-receipt.json')
    for row in json.loads(INPUTS.read_text())['files']:
        assert sha(row['path']) == row['sha256'], row['path']
    assert sha(J458/'resource-receipt.json') == RECEIPT458_SHA and not (J458/'preview458-summary.json').exists()
    assert listing(J458) == LISTING458_SHA, 'preview458 changed since task 516'
    chapters = subprocess.check_output([str(BINARY), 'chapters'], cwd=R, text=True, timeout=120).splitlines()
    assert len(chapters) == len(set(chapters)) == 15 and tuple(chapters[:8]) == DONE, chapters
    qa = STEMS/'qa'
    assert {p.stem for p in qa.glob('*.json')} == set(chapters)
    assert all(json.loads((qa/(c+'.json')).read_text())['passed'] is True for c in chapters)
    record = {c: chapter(J458/'preview', c) for c in DONE}
    todo = chapters[8:]
    assert not OUT.exists(), OUT
    if check_only:
        mine, limit = pressure()
        say(check_only='passed', reused=list(DONE), to_render=todo, processes=mine, limit=limit)
        return 0
    render_all(todo, record)
    assert listing(J458) == LISTING458_SHA, 'preview458 changed during recovery528'
    with (JOB/'preview528-summary.json').open('x') as f:
        json.dump(dict(chapters=chapters, all_timelines_byte_identical=True, guard=guard, **hashes,
                       reused=dict(job=str(J458.relative_to(R)), receipt_sha256=RECEIPT458_SHA,
                                   listing_sha256=LISTING458_SHA, chapters=list(DONE)),
                       rendered=dict(folder=str(OUT.relative_to(R)), chapters=todo), chapter=record,
                       x264=X264, encoders=ENCODERS,
                       scope='540p narrated review candidate only: 01-08 are preview458 files reused by hash; 09-15 '
                             'rendered by recovery528 with the same binary, inputs and flags. Actual pictures, motion, '
                             'listening, fine sync, independent final AV and release unaccepted'), f, indent=2)
    return 0


if __name__ == '__main__':
    raise SystemExit(main(sys.argv[1:]))
