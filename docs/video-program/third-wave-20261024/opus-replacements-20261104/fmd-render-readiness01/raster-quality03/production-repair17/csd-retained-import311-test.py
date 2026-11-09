"""Source-level tests for csd-retained-import311.py. Nothing here touches a host or runs a job.

Run from anywhere:  python3 -I -B csd-retained-import311-test.py --tmp DIR
It reads files in its own folder (the helper, cc0-combined-master293.py, the retained workers, the
actual closure manifests final13-export97.json and final15-export56.json, and the local copies of
the retained finals' records). It writes only inside a new folder it creates under DIR, and it
deletes nothing.

What it covers:
  pins        the helper's chapter pins equal 293 RETAINED, and its host constants equal the three
              retained workers' own constants;
  records     the receipt, summary, inputs, permit, worker and movie pins equal the local copies of
              the terminal records;
  closure     the actual CSS and HZ2 closure manifests pass the helper's manifest and closure checks,
              reading every record that has a local copy; every shared D row equals the CSD hash
              recorded in local CSD evidence;
  trj         the 02/03 closure checks run on the TRJ records available here, and the files they
              still lack are exactly the ones no local copy holds;
  mutations   each kind of broken or hostile manifest is refused, with the reason named;
  stream      header, staging, placement and alias on a scratch tree that stands in for CSD, with
              the module's root constants pointed into it: round trip, re-run, truncated stream,
              trailing data, corrupted bytes, wrong header, divergent file, existing roots, missing
              shared file, a racing writer and a symlink inside the physical root;
  source      the helper only writes with 'x'/'xb', makes one hard link per new file and one alias,
              never renames, replaces or removes a file (staged names are kept), and starts no
              process.
The stream tests show the code paths only. They are not an import on CSD.
"""
import argparse, copy, hashlib, importlib.util, io, json, os, re, stat, sys, tempfile
from pathlib import Path

Q = Path(__file__).resolve().parent
HELPER = Q/'csd-retained-import311.py'
TESTS = []


def test(fn):
    TESTS.append(fn)
    return fn


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


H = load('importer311', HELPER)
LOCAL = {
    '02_turning_numbers': dict(dir='final02-terminal167', permit='unchanged-preview16-review30/final-render-permit32.json'),
    '03_two_planes': dict(dir='final03-terminal115', permit='preview21-03-review/final-render-permit28.json'),
    '13_real_integral': dict(dir='final13-terminal200', permit='preview21-13-review95/final-render-permit96.json',
                             soundtrack='final13-terminal200/soundtrack_0.wav'),
    '15_closing': dict(dir='final15-terminal81', tables='retained15-tables281', permit='hz2-review15-53/final-render-permit53.json'),
}
WORKERS = {'trj': 'trj-final-chapter19-02.py', 'css': 'css-final-chapter78.py', 'hz2': 'hz2-final-chapter53.py'}
EXPORTS = {'css': 'final13-export97.json', 'hz2': 'final15-export56.json'}


def csd_records():
    """Exact bytes of CSD files reproduced from local CSD evidence (each checked by hash where used)."""
    score = json.loads((Q/'score21-terminal01.json').read_text())
    audit = json.loads((Q/'timeline21-terminal01/audit.json').read_text())
    data = {H.SCORE21/'score21-summary.json': json.dumps(score['summary'], indent=2).encode(),
            H.SCORE21/'resource-receipt.json': (json.dumps(score['receipt'], indent=2)+'\n').encode(),
            H.TIMELINE21/'resource-receipt.json': (json.dumps(audit['receipt'], indent=2)+'\n').encode(),
            H.BUILD19/'qualification.json': (Q/'final-picture-source41/000-qualification.json').read_bytes()}
    for p in (Q/'timeline21-terminal01').glob('*.tsv'):
        data[H.TIMELINE21/'timeline-output'/p.name] = p.read_bytes()
    return data, score['summary']['files']


def local_files(ch):
    """Remote path -> local copy for one retained chapter's records."""
    pin = H.CHAPTERS[ch]
    job = H.HOSTS[pin['host']]['root']/'jobs'/pin['job']
    render = job/'render'
    spec = LOCAL[ch]
    d, t = Q/spec['dir'], Q/spec.get('tables', spec['dir'])
    out = {job/'resource-receipt.json': d/'resource-receipt.json', job/'render-summary.json': d/'render-summary.json',
           pin['permit']: Q/spec['permit'], pin['worker']: Q/WORKERS[pin['host']],
           render/(ch+'.mp4'): d/(ch+'.mp4'), render/(ch+'.layout.tsv'): d/(ch+'.layout.tsv')}
    for kind in H.KINDS:
        out[render/(ch+'.'+kind+'.tsv')] = t/(ch+'.'+kind+'.tsv')
    if 'soundtrack' in spec:
        out[render/'soundtrack'/ch/'soundtrack_0.wav'] = Q/spec['soundtrack']
    return out


def reader_for(host, override=None):
    data, _ = csd_records()
    files = {}
    for ch in H.HOSTS[host]['chapters']:
        files.update(local_files(ch))
    override = override or {}

    def reader(path):
        path = Path(path)
        if path in override:
            return override[path]
        if path in data:
            return data[path]
        if path in files:
            return files[path].read_bytes()
        raise H.Unavailable(str(path))
    return reader


def actual(host):
    return json.loads((Q/EXPORTS[host]).read_text())


def manifest(host):
    return H.manifest_from_rows(host, actual(host)['files'])


# ---------------------------------------------------------------- pins

@test
def pins_equal_293_retained():
    m = load('master293_for_311_test', Q/'cc0-combined-master293.py')
    assert set(H.CHAPTERS) == set(m.RETAINED) and set(H.CHAPTERS).isdisjoint(m.CHANGED)
    for ch, r in m.RETAINED.items():
        c = H.CHAPTERS[ch]
        assert H.HOSTS[c['host']]['root'] == r['host'] and ch in H.HOSTS[c['host']]['chapters'], ch
        for k in ('worker', 'job', 'inputs', 'permit', 'permit_sha256', 'movie_sha256', 'frames'):
            assert c[k] == r[k], (ch, k)
    assert (H.D, H.Q, H.GUARD, H.PYTHON, H.BINARY_19, H.TRJ, H.CSS, H.HZ2) == \
        (m.D, m.Q, m.GUARD, m.PYTHON, m.BINARY_19, m.TRJ, m.CSS, m.HZ2)
    assert (H.TIMELINE21, H.SCORE21, H.STEMS, H.BUILD19) == (m.BASELINE.parent, m.SCORE, m.STEMS, m.BUILD19)
    for host in H.HOSTS:                              # what 293 imported() accepts
        job = H.job_name(host, 1)
        receipt, inputs = m.D/'jobs'/job/'resource-receipt.json', H.inputs_path(job)
        assert receipt.parent.parent == m.D/'jobs' and inputs.is_relative_to(m.D)
        assert re.fullmatch('[a-z0-9_-]+', job), job      # the guard's job-name rule
    return '4 chapters, 7 fields each, equal; import receipts and inputs fit 293 imported()'


@test
def host_constants_equal_workers():
    for host, name in WORKERS.items():
        text = (Q/name).read_text()
        w = load('worker_'+host, Q/name)
        spec = H.HOSTS[host]
        assert (w.ROOT, w.NARRATION, w.D, w.BIN, w.BINARY_SHA, w.STEMS) == (spec['root'], spec['narration'], H.D, H.BIN19, H.BINARY_19, H.STEMS)
        assert w.TIMELINES == H.TIMELINE21/'timeline-output'
        check = text[text.index('def check('):text.index('def verify_tables(')]
        receipts = [w.ROOT/x for x in re.findall(r"ROOT/'(jobs/[^']+/resource-receipt\.json)'", check)]
        assert tuple(receipts) == spec['checks'], host
        shared = [w.D/x for x in re.findall(r"D/'(jobs/[^']+)'", check)]
        assert set(shared) <= set(H.SHARED_READS) | {H.SCORE21/'score'}, sorted(map(str, shared))
        inherited = re.findall(r"inherited=ROOT/'([^']+)'", text)
        assert [w.ROOT/x for x in inherited] == [spec['delivery_inputs']], host
        for ch in spec['chapters']:
            assert sha(Q/name) == H.CHAPTERS[ch]['worker_sha256'], ch
    return 'TRJ, CSS, HZ2 workers: root, narration, check() receipts, delivery inputs, D constants'


@test
def terminal_records_equal_pins():
    for ch, pin in H.CHAPTERS.items():
        files = local_files(ch)
        job = H.HOSTS[pin['host']]['root']/'jobs'/pin['job']
        receipt, summary = files[job/'resource-receipt.json'], files[job/'render-summary.json']
        assert sha(receipt) == pin['receipt_sha256'] and sha(summary) == pin['summary_sha256'], ch
        r = json.loads(receipt.read_text())
        assert H.terminal(r) and r['inputs_sha256'] == pin['inputs_sha256'], ch
        assert sha(files[pin['permit']]) == pin['permit_sha256'], ch
        assert sha(files[job/'render'/(ch+'.mp4')]) == pin['movie_sha256'], ch
        review = json.loads(files[pin['permit']].read_text())
        assert (review['chapter'], review['frames'], len(review['evidence'])) == (ch, pin['frames'], pin['evidence']), ch
    for host, name in EXPORTS.items():
        a = actual(host)
        (ch,) = H.HOSTS[host]['chapters']
        pin = H.CHAPTERS[ch]
        assert a['receipt_sha256'] == pin['receipt_sha256'], host
        assert a['source_job'] == str(H.HOSTS[host]['root']/'jobs'/pin['job']), host
    return 'receipt, summary, inputs, permit and movie pins equal the local terminal records'


# ---------------------------------------------------------------- actual closures

@test
def actual_closures_pass():
    out = []
    for host, counts in (('css', (859, 52)), ('hz2', (855, 52))):
        m = manifest(host)
        assert H.manifest_problems(m) == [], H.manifest_problems(m)
        assert (m['foreign_files'], m['shared_files']) == counts, host
        assert m['foreign_bytes']+m['shared_bytes'] == actual(host)['bytes']
        problems, unavailable = H.closure_problems(m, reader_for(host))
        assert problems == [], problems
        spec = H.HOSTS[host]
        lacking = set(map(str, spec['checks'])) | {str(spec['delivery_inputs'])} | {str(H.CHAPTERS[c]['inputs']) for c in spec['chapters']}
        assert set(unavailable) == lacking, sorted(set(unavailable) ^ lacking)
        out.append('%s %d foreign files %.2f GiB (measured growth %.2f, limit %.2f), %d shared; unread here: %d host receipts/inputs' % (
            host, m['foreign_files'], m['foreign_bytes']/H.GIB, 2*m['foreign_bytes']/H.GIB, H.GROWTH_LIMIT/H.GIB, m['shared_files'], len(unavailable)))
    return '; '.join(out)


@test
def shared_rows_equal_csd_records():
    data, score_files = csd_records()
    out = []
    for host in EXPORTS:
        m = manifest(host)
        matched = 0
        for x in m['files']:
            if x['kind'] != 'shared':
                continue
            p = Path(x['path'])
            if p == H.BIN19:
                assert x['sha256'] == H.BINARY_19
            elif p in data:
                assert H.sha_bytes(data[p]) == x['sha256'], str(p)
            else:
                assert score_files[str(p.relative_to(H.SCORE21/'score'))] == x['sha256'], str(p)
            matched += 1
        out.append('%s %d/%d' % (host, matched, m['shared_files']))
    return 'shared D rows equal to CSD-recorded hashes: '+', '.join(out)


@test
def trj_records_closure():
    """No TRJ export exists locally. Build the TRJ rows the local records name, then check that what
    the closure check still lacks is exactly what no local copy holds."""
    data, score_files = csd_records()
    rows = {}

    def add(path, digest, size=0, mode=0o644):
        rows.setdefault(str(path), dict(path=str(path), sha256=digest, bytes=size, mode=mode))
    for path, digest in data.items():
        add(path, H.sha_bytes(digest), len(digest))
    add(H.BIN19, H.BINARY_19)
    for relative, digest in score_files.items():
        add(H.SCORE21/'score'/relative, digest)
    evidence = 0
    for ch in H.HOSTS['trj']['chapters']:
        for remote, local in local_files(ch).items():
            add(remote, sha(local), local.stat().st_size)
        pin = H.CHAPTERS[ch]
        job = H.TRJ/'jobs'/pin['job']
        summary = json.loads(local_files(ch)[job/'render-summary.json'].read_text())
        for x in summary['files']:
            add(x['path'], x['sha256'])
        review = json.loads((Q/LOCAL[ch]['permit']).read_text())
        for item in review['evidence']:
            assert Path(item['path']).is_relative_to(H.TRJ)
            add(item['path'], item['sha256'])
            evidence += 1
        for line in local_files(ch)[job/'render'/(ch+'.cuelog.tsv')].read_text().splitlines():
            if line.strip() and not line.startswith('#'):
                row = line.split('\t')
                add(row[0], row[1], int(row[2]))
    m = H.manifest_from_rows('trj', sorted(rows.values(), key=lambda x: x['path']))
    assert H.manifest_problems(m) == [], H.manifest_problems(m)
    problems, unavailable = H.closure_problems(m, reader_for('trj'))
    assert unavailable == [], unavailable
    lacks = {p.split(': ', 1)[1] for p in problems if p.startswith('closure lacks ')}
    others = [p for p in problems if not p.startswith('closure lacks ')]
    expected = {str(H.CHAPTERS[c]['inputs']) for c in H.HOSTS['trj']['chapters']} | \
        {str(H.HOSTS['trj']['delivery_inputs'])} | set(map(str, H.HOSTS['trj']['checks']))
    assert lacks == expected, sorted(lacks ^ expected)
    assert len(others) == 1 and others[0].startswith('narration folder holds '), others
    wavs = sum(1 for p in rows if Path(p).parent == H.HOSTS['trj']['narration'])
    return ('02+03: %d evidence items, %d narration WAVs, render files and receipts all checked; still lacking only '
            'the 2 render inputs, delivery02 inputs and receipt (TRJ copies, none local)' % (evidence, wavs))


# ---------------------------------------------------------------- refusals

def refused(m, needle, reader=None, host='css'):
    problems = H.manifest_problems(m)
    if reader is not False:
        try:
            more, _ = H.closure_problems(m, reader or reader_for(host))
        except (KeyError, TypeError, AttributeError) as error:
            more = ['closure check raised '+repr(error)]
        problems = problems+more
    assert any(needle in p for p in problems), (needle, problems[:5])
    return True


@test
def mutations_refused():
    base = manifest('css')
    pin = H.CHAPTERS['13_real_integral']
    job = H.CSS/'jobs'/pin['job']
    review = json.loads((Q/LOCAL['13_real_integral']['permit']).read_text())
    rows = {x['path']: i for i, x in enumerate(base['files'])}
    cases = []

    def case(name, change, needle, reader=None):
        m = copy.deepcopy(base)
        change(m)
        assert refused(m, needle, reader), name
        cases.append(name)

    def drop(path):
        return lambda m: m['files'].pop(next(i for i, x in enumerate(m['files']) if x['path'] == str(path)))

    def row(m, path):
        return next(x for x in m['files'] if x['path'] == str(path))
    wav = H.HOSTS['css']['narration']/'13_1.wav'
    movie = job/'render/13_real_integral.mp4'
    case('missing permit evidence', drop(review['evidence'][0]['path']), 'closure lacks permit evidence')
    case('missing narration WAV', drop(wav), 'closure lacks')
    case('missing narration WAV count', drop(wav), 'narration folder holds 104 WAVs')
    case('missing check() receipt', drop(H.CSS/'jobs/css-encoder-stop79/resource-receipt.json'), 'closure lacks check() receipt')
    case('missing timeline21 table', drop(H.TIMELINE21/'timeline-output/13_real_integral.cues.tsv'), 'closure lacks timeline21 table')
    case('missing worker', drop(pin['worker']), 'closure lacks worker')
    case('missing job file', drop(job/'render/soundtrack/13_real_integral/soundtrack_0.wav'), 'closure lacks render output')
    case('wrong movie hash', lambda m: row(m, movie).update(sha256='0'*64), 'render output is 000000000000')
    case('wrong evidence hash', lambda m: row(m, review['evidence'][0]['path']).update(sha256='1'*64), 'permit evidence is 111111111111')
    case('summary swapped', lambda m: row(m, job/'render-summary.json').update(sha256='2'*64), 'render summary is 222222222222')
    case('altered receipt bytes', lambda m: None, 'bytes of render receipt differ',
         reader_for('css', {job/'resource-receipt.json': b'{}'}))
    case('foreign root of another host', lambda m: m['files'].append(dict(row(m, movie), path=str(H.TRJ/'x.bin'), kind=None)), 'row outside')
    case('D row in the stream', lambda m: row(m, H.BIN19).update(kind='foreign'), "row kind 'foreign' is not 'shared'")
    case('host row marked shared', lambda m: row(m, movie).update(kind='shared'), "row kind 'shared' is not 'foreign'")
    case('dot-dot path', lambda m: m['files'].append(dict(row(m, movie), path=str(H.CSS)+'/jobs/../x')), 'not a clean absolute path')
    case('relative path', lambda m: m['files'].append(dict(row(m, movie), path='jobs/x')), 'not a clean absolute path')
    case('duplicate row', lambda m: m['files'].append(dict(row(m, movie))), 'duplicate row')
    case('row in the physical root', lambda m: m['files'].append(dict(row(m, movie), path=str(H.physical_root('css')/'x'), kind='shared')),
         'row inside a 311 import')
    case('row in an import job', lambda m: m['files'].append(dict(row(m, movie), path=str(H.D/'jobs/csd-retained-import311-css-a1/x'), kind='shared')),
         'row inside a 311 import')
    case('file under a file', lambda m: m['files'].append(dict(row(m, movie), path=str(movie)+'/x')), 'a row is also a directory')
    case('setuid mode', lambda m: row(m, movie).update(mode=0o4755), 'bad mode')
    case('unreadable mode', lambda m: row(m, movie).update(mode=0o044), 'bad mode')
    case('bad digest', lambda m: row(m, movie).update(sha256='XYZ'), 'bad sha256')
    case('extra key', lambda m: row(m, movie).update(note='x'), 'malformed row')
    case('pin changed', lambda m: m['chapters']['13_real_integral'].update(frames=10642), 'chapter pins differ')
    case('wrong root', lambda m: m.update(root=str(H.TRJ)), 'root is not')
    case('wrong totals', lambda m: m.update(foreign_bytes=m['foreign_bytes']+1), 'totals differ')
    case('over the growth limit', lambda m: (row(m, movie).update(bytes=4*H.GIB), m.update(foreign_bytes=m['foreign_bytes']+4*H.GIB-row(base, movie)['bytes'])),
         'exceeds the per-job growth limit', False)
    case('wrong version', lambda m: m.update(version='310'), 'not a version-311 manifest', False)
    case('unknown host', lambda m: m.update(host='ts1'), 'unknown host', False)
    return '%d refusals: %s' % (len(cases), ', '.join(cases))


# ---------------------------------------------------------------- stream, staging, placement

class World:
    """A scratch tree standing in for CSD: D with jobs/ and work/, an absent foreign root beside it,
    and a source tree that send() reads before it is moved aside."""

    def __init__(self, base, name):
        self.base = base/name
        projects = self.base/'projects'
        self.m = load('importer311_'+name, HELPER)
        self.m.D = projects/'D'
        self.root = projects/'TRJ'
        self.m.HOSTS = {'trj': dict(H.HOSTS['trj'], root=self.root)}
        (self.m.D/'jobs').mkdir(parents=True)
        (self.m.D/'work').mkdir()
        self.shared = self.m.D/'jobs/shared/table.tsv'
        self.shared.parent.mkdir()
        self.shared.write_bytes(b'shared bytes\n')
        self.top = self.m.physical_root('trj')
        self.contents = {'qualification01/a.json': (b'{"a": 1}\n', 0o644), 'jobs/j/render/b.bin': (os.urandom(3*(1 << 20)+17), 0o600),
                         'jobs/j/render/soundtrack/c.wav': (b'', 0o664), 'work/frozen/x/y/z.py': (b'print(1)\n', 0o755)}

    def source(self):
        rows = []
        for relative, (data, mode) in sorted(self.contents.items()):
            p = self.root/relative
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(data)
            os.chmod(p, mode)
            rows.append(dict(path=str(p), sha256=hashlib.sha256(data).hexdigest(), bytes=len(data), mode=mode))
        rows.append(dict(path=str(self.shared), sha256=sha(self.shared), bytes=self.shared.stat().st_size, mode=0o644))
        m = self.m.manifest_from_rows('trj', rows)
        assert self.m.manifest_problems(m) == [], self.m.manifest_problems(m)
        path = self.base/'manifest.json'
        path.write_text(json.dumps(m))
        return m, path, sha(path)

    def send(self, path, digest):
        out = io.BytesIO()

        class Stdout:
            buffer = out
        saved, sys.stdout = sys.stdout, Stdout()
        try:
            assert self.m.send(argparse.Namespace(manifest=path, manifest_sha=digest, host='trj')) == 0
        finally:
            sys.stdout = saved
        os.rename(self.root, self.base/'source-moved-aside')     # the source host's tree, kept
        return out.getvalue()

    def receive(self, m, digest, stream, job='job-a1'):
        plan, problems = self.m.plan_problems(m)
        assert problems == [], problems
        s = io.BufferedReader(io.BytesIO(stream))
        self.m.read_header(s, self.m.header_fields(m, digest))
        incoming = self.m.D/'jobs'/job/'incoming'
        incoming.mkdir(parents=True)
        sources = self.m.stage(s, plan, incoming)
        assert not self.top.exists() or job != 'job-a1', 'staging wrote outside the job'
        rows = self.m.place(plan, sources, 'trj')
        return plan, rows, self.m.make_alias('trj'), incoming


def expect(error, fn, needle):
    try:
        fn()
    except error as e:
        assert needle in str(e), (needle, str(e))
        return
    raise AssertionError('not refused: '+needle)


@test
def stream_round_trip_and_rerun(tmp):
    w = World(tmp, 'roundtrip')
    m, path, digest = w.source()
    stream = w.send(path, digest)
    assert stream.startswith(H.MAGIC+b' ') and len(stream.split(b'\n', 1)[1]) == m['foreign_bytes']
    assert not w.root.exists() and not w.top.exists()
    plan, rows, alias, incoming = w.receive(m, digest, stream)
    assert alias and w.root.is_symlink() and os.readlink(w.root) == str(w.top)
    assert sum(r['written'] for r in rows) == 4 and len(list(incoming.iterdir())) == 4
    staged = {r['path']: Path(r['staged']) for r in rows}
    for relative, (data, mode) in w.contents.items():
        p = w.root/relative
        assert p.read_bytes() == data and (p.stat().st_mode & 0o777) == mode, relative
        kept = staged[str(w.root/relative)]
        assert kept.parent == incoming and kept.read_bytes() == data, 'staged name not kept: '+relative
        assert os.path.samefile(kept, w.top/relative) and (w.top/relative).stat().st_nlink == 2, 'not one inode with two names: '+relative
    plan2, rows2, alias2, incoming2 = w.receive(m, digest, stream, job='job-a2')
    assert not alias2 and not any(r['written'] for r in rows2) and all(not new for _, _, new in plan2)
    assert list(incoming2.iterdir()) == [] and len(list(incoming.iterdir())) == 4

    def allocated(trees):
        """The guard's allocated(): st_blocks of regular files, hard links counted once per call."""
        seen, total = set(), 0
        for tree in trees:
            for parent, dirs, files in os.walk(tree, followlinks=False):
                for name in files:
                    st = os.lstat(Path(parent)/name)
                    if stat.S_ISREG(st.st_mode) and (st.st_dev, st.st_ino) not in seen:
                        seen.add((st.st_dev, st.st_ino))
                        total += st.st_blocks*512
        return total
    jobs, work, both = allocated([incoming]), allocated([w.top]), allocated([incoming, w.top])
    assert jobs == work == both > 0, (jobs, work, both)
    return ('send -> stage -> place -> alias; every staged name kept as a second link to the placed inode, modes kept; '
            're-run writes nothing; guard-style growth %d+%d bytes for %d bytes of disk blocks' % (jobs, work, both))


@test
def stream_refusals(tmp):
    w = World(tmp, 'refusals')
    m, path, digest = w.source()
    stream = w.send(path, digest)
    plan, problems = w.m.plan_problems(m)
    assert problems == []
    head, body = stream.split(b'\n', 1)
    fields = w.m.header_fields(m, digest)
    for name, data, needle in (
            ('wrong manifest hash', H.MAGIC+b' '+json.dumps(dict(fields, manifest_sha256='0'*64)).encode()+b'\n', 'names another manifest'),
            ('wrong count', H.MAGIC+b' '+json.dumps(dict(fields, files=3)).encode()+b'\n', 'names another manifest'),
            ('wrong magic', b'CSD-RETAINED-IMPORT310 '+head.split(b' ', 1)[1]+b'\n', 'not a 311 import stream'),
            ('no header line', b'x'*(H.HEADER_MAX+10), 'does not start with a header line')):
        expect(AssertionError, lambda: w.m.read_header(io.BufferedReader(io.BytesIO(data)), fields), needle)
    for name, data, needle in (('truncated', body[:-5], 'truncated stream'), ('trailing', body+b'!', 'unexpected data'),
                               ('corrupted', body[:100]+bytes([body[100] ^ 1])+body[101:], 'stream bytes differ')):
        incoming = w.m.D/'jobs'/('job-'+name)/'incoming'
        incoming.mkdir(parents=True)
        expect(AssertionError, lambda: w.m.stage(io.BufferedReader(io.BytesIO(data)), plan, incoming), needle)
        assert not w.top.exists() and not w.root.exists(), 'a refused stream wrote outside its job'
    return 'wrong hash, count or magic, no header, truncated, trailing and corrupted streams refused before any placement'


@test
def destination_refusals(tmp):
    out = []
    w = World(tmp, 'divergent')
    m, path, digest = w.source()
    w.send(path, digest)
    victim = w.top/'qualification01/a.json'
    victim.parent.mkdir(parents=True)
    victim.write_bytes(b'different\n')
    _, problems = w.m.plan_problems(m)
    assert any('existing destination differs' in p for p in problems) and victim.read_bytes() == b'different\n'
    out.append('divergent file kept')
    w = World(tmp, 'realroot')
    m, path, digest = w.source()
    _, problems = w.m.plan_problems(m)
    assert any('foreign root exists as a real path' in p for p in problems)
    out.append('real foreign root')
    w = World(tmp, 'otheralias')
    m, path, digest = w.source()
    w.send(path, digest)
    (w.base/'elsewhere').mkdir()
    w.root.symlink_to(w.base/'elsewhere', target_is_directory=True)
    _, problems = w.m.plan_problems(m)
    assert any('symlink to somewhere else' in p for p in problems)
    out.append('alias to elsewhere')
    w = World(tmp, 'shared')
    m, path, digest = w.source()
    w.send(path, digest)
    w.shared.write_bytes(b'changed\n')
    _, problems = w.m.plan_problems(m)
    assert any('shared CSD file missing or different' in p for p in problems)
    out.append('changed shared file')
    w = World(tmp, 'symlinkdir')
    m, path, digest = w.source()
    w.send(path, digest)
    (w.base/'outside').mkdir()
    w.top.mkdir()
    (w.top/'jobs').symlink_to(w.base/'outside', target_is_directory=True)
    _, problems = w.m.plan_problems(m)
    assert any('not a real directory' in p for p in problems)
    out.append('symlink inside the physical root')
    w = World(tmp, 'race')
    m, path, digest = w.source()
    stream = w.send(path, digest)
    plan, problems = w.m.plan_problems(m)
    s = io.BufferedReader(io.BytesIO(stream))
    w.m.read_header(s, w.m.header_fields(m, digest))
    incoming = w.m.D/'jobs/job-race/incoming'
    incoming.mkdir(parents=True)
    sources = w.m.stage(s, plan, incoming)
    racer = w.top/'jobs/j/render/b.bin'
    racer.parent.mkdir(parents=True)
    racer.write_bytes(b'racer\n')
    expect(FileExistsError, lambda: w.m.place(plan, sources, 'trj'), '')
    assert racer.read_bytes() == b'racer\n' and not w.root.exists()
    out.append('racing writer never overwritten, no alias')
    w = World(tmp, 'attempts')
    first = w.m.D/'jobs'/w.m.job_name('trj', 1)
    first.mkdir()
    (first/'resource-receipt.json').write_text(json.dumps(dict(admitted=True)))
    assert any('still running' in p for p in w.m.earlier_attempts('trj', 2))
    second = w.m.D/'jobs'/w.m.job_name('trj', 2)
    second.mkdir()
    (second/'resource-receipt.json').write_text(json.dumps(dict(finished='t', exit=0, inputs_sha256='a', final_inputs_sha256='a')))
    (second/'import-result.json').write_text('{}')
    assert any('already imported' in p for p in w.m.earlier_attempts('trj', 3))
    out.append('running or finished earlier attempt')
    return 'refused before any write: '+', '.join(out)


@test
def commands_bind_the_job():
    helper = H.Q/'csd-retained-import311.py'
    manifest = H.Q/'retained-import311-css-manifest.json'
    command = H.guard_command(helper, 'css', 1, manifest)
    cut = command.index('--')
    assert command[:cut] == [H.PYTHON, '-I', '-B', str(H.GUARD), '--phase', 'native', '--job', 'csd-retained-import311-css-a1',
                             '--inputs', str(H.Q/'csd-retained-import311-css-a1-inputs.json')]
    assert command[cut+1:] == H.receive_command(helper, 'css', 1, manifest)
    return 'guard command = guard argv + "--" + the exact receive argv the receipt must record'


@test
def helper_source_writes_only_new_paths():
    text = HELPER.read_text()
    code = text.split('"""', 2)[2]
    for banned in ('rmtree', 'rmdir', 'os.remove', 'os.rename', 'os.replace', 'unlink', 'write_text', 'write_bytes', 'subprocess',
                   "open('w", "open('a", "open('r+", 'shutil', 'symlink_to(', '.truncate(', 'chown', 'os.link('):
        allowed = {'symlink_to(': 1, 'os.link(': 1}.get(banned, 0)
        assert code.count(banned) == allowed, (banned, code.count(banned))
    opens = re.findall(r"open\('([a-z+]+)'\)", code)
    assert set(opens) <= {'x', 'xb', 'rb'}, opens
    return "writes only with 'x'/'xb'; one hard link (placement), one symlink (alias); removes, renames and replaces nothing; no process"


def main():
    p = argparse.ArgumentParser(description=__doc__.split('\n')[0])
    p.add_argument('--tmp', type=Path, required=True, help='an existing folder; a new subfolder is made in it')
    args = p.parse_args()
    tmp = Path(tempfile.mkdtemp(prefix='import311-test-', dir=args.tmp))
    failed = 0
    for fn in TESTS:
        try:
            note = fn(tmp) if fn.__code__.co_argcount else fn()
            print('PASS', fn.__name__, '-', note)
        except Exception as error:                        # report every test, then fail
            failed += 1
            print('FAIL', fn.__name__, '-', type(error).__name__, str(error)[:600])
    print('%d/%d passed; scratch tree %s' % (len(TESTS)-failed, len(TESTS), tmp.name))
    return 1 if failed else 0


if __name__ == '__main__':
    raise SystemExit(main())
