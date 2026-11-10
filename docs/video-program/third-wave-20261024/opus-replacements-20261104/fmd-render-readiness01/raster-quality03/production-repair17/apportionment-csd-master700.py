"""Guarded CSD native master for apportionment: chapters 01-02 as transported by import683, 03-15 from final672.

Successor of root's master636 for the mixed-host case. Prepared for root; its author never executed it. On the CSD only,
with /usr/bin/python3 -I -B:
  inputs   no job: requires stable successful import683 and all 13 final672 runs, repeats every check that writes nothing,
           and writes one new guard manifest.
  run      only as the child of guard_csd02: rechecks everything, builds final/ inside the new job folder, then runs the
           native master and captions commands once.

What differs from master636, and why:
  1. Chapters 01-02 resolve through import683's path -> physical records; nothing is guessed from a path name.
  2. Their timeline and cue-log tables name Mac paths, and the kit opens those paths (master.rs replay,
     narration_delivery.py timeline and cue_log). final/ therefore holds a copy with the path column alone rebased, and
     the copy must equal timeline666's own table byte for byte (the cue log: timeline666's rows plus the exact stem rows).
     The imported files are kept and never edited.
  3. master gets --score. master636 omitted it, and narration_delivery.cue_log refuses a stem row without it (lines 153-180).
  4. Encoder settings: raw equality within each host's chapters, as master636 required of all 15, and across hosts the
     options must match x264's reference with only support671's two exemptions (build banner, auto thread counts). Each
     group's encoder tags are bound to the ffmpeg that made it.
  5. Chapters 01-02 go through support671's whole chapter() on this host, so they meet this ffprobe and this decoder. 03-15
     passed the same function here in their own final672 runs and are re-bound by hash, as master636 did.

Every recording stays the exact source file. No check is relaxed for the host: the kit's replay must still equal each
captured soundtrack bit for bit, so if Mac and CSD mixers differ this job stops, and that is the correct result.

This accepts no picture, motion, audio, fine-sync, AV or release result. verify701 is a separate later job.
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
from pathlib import Path

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
JOBS = D/'jobs'
A, BUILD, T = JOBS/'csd-apportionment-assembly664', JOBS/'csd-apportionment-build656', JOBS/'csd-apportionment-timeline666'
IMPORT = JOBS/'csd-apportionment-import683-a1'
JOB = JOBS/'csd-apportionment-master700'
INPUTS = Q/'csd-apportionment-master700-inputs.json'
HELPER = Path(__file__).resolve()
CONTROLLER = Q/'controller702.py'
SUPPORT = Q/'apportionment-csd-support671.py'
SUPPORT_SHA = '244fe1a47d5a40f71594031bafc09fefb45558c38ed72adbfb6aa1b1f269c40a'
EXPORT_SHA = '1c7c690b603ed0a3571ba27e033dc2c1af581cacf74b171e01996e702e2285c8'
REL = 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
NARRATION = A/'repo'/REL/'apportionment-picture-assembly566/narration'
STEMS = A/'repo'/REL/'apportionment-frederick-score445/score/stems'
BINARY = BUILD/'apportionment-native656'
MAC_TAGS = ('Lavc62.28.102 libx264', 'Lavf62.12.102')
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4, work_cap_gib=56, target_growth_cap_gib=20,
              sample_seconds=10)
PYTHON = '/usr/bin/python3'
FFMPEG, FFPROBE = '/usr/bin/ffmpeg', '/usr/bin/ffprobe'
TOTAL_FRAMES = 100886
ROWS_683 = 4005
TIMELINE_KINDS = ('chapter', 'fps', 'frames', 'mark', 'breath')
SCOPE = ('Native master replay, frame grid and caption commands passed. Chapters 01-02 were encoded on the Mac and 03-15 on '
         'the CSD. Separate final decode, caption, loudness, picture, motion, audio, fine-sync and independent AV checks '
         'remain; nothing here accepts any of them.')


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


def stable(paths, seconds=15):
    first = {p: sha(p) for p in paths}
    time.sleep(seconds)
    assert first == {p: sha(p) for p in paths}, 'a terminal record changed while being read'


def libraries(path):
    """The shared libraries a binary loads, as ldd resolves them; an unresolved one refuses."""
    out = subprocess.check_output(['/usr/bin/ldd', str(path)], text=True, timeout=60, env=dict(PATH='/usr/bin:/bin'))
    assert 'not found' not in out, path
    return {Path(m[1]).resolve() for m in (re.search(r'(/\S+)', line) for line in out.splitlines()) if m}


def load():
    """Support671 by its pin. Its root is explicit (no parents[N]), so it imports here unedited, as final672 does."""
    assert sha(SUPPORT) == SUPPORT_SHA, 'support671 changed'
    spec = importlib.util.spec_from_file_location('owned671', SUPPORT)
    o = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(o)
    assert tuple(o.ENCODERS) == MAC_TAGS, 'support671 no longer carries the Mac encoder tags'
    o.R, o.REF, o.STEMS, o.FFMPEG, o.FFPROBE = D, T/'timeline', STEMS, FFMPEG, FFPROBE
    return o


def csd_encoder():
    """The ffmpeg -version text and the stream and container tags it writes, derived exactly as final672 derives them."""
    version = subprocess.check_output([FFMPEG, '-version'], text=True, timeout=30)
    def lib(name):
        found = re.search(r'^'+name+r'\s+(\d+)\.\s*(\d+)\.\s*(\d+)\s*/', version, re.M)
        assert found, name
        return '.'.join(found.groups())
    return version, ('Lavc'+lib('libavcodec')+' libx264', 'Lavf'+lib('libavformat'))


def options(value):
    """x264 options with only the two things support671's normalized_encoder exempts: build banner, auto thread counts."""
    for key in ('threads', 'lookahead_threads'):
        found = re.findall(r'(?<![a-z_])'+key+r'=(\d+)', value)
        assert len(found) == 1 and 1 <= int(found[0]) <= 128, (key, found)
        value = re.sub(r'(?<![a-z_])'+key+r'=\d+', key+'=AUTO', value)
    assert value.startswith('x264 - core ') and ' - options: ' in value
    return value.split(' - options: ', 1)[1]


def imported(o):
    """Successful import683 and its path -> physical records, tied row by row to export682."""
    receipt, result = IMPORT/'resource-receipt.json', IMPORT/'import-result.json'
    o.success(receipt)
    data = read(result)
    manifest = Path(data['manifest'])
    assert manifest.parent == Q and data['manifest_sha256'] == EXPORT_SHA == sha(manifest), 'import683 carried another manifest'
    export = read(manifest)
    rows = data['files']+data['reused_files']
    mapping = {r['path']: r for r in rows}
    assert len(mapping) == len(rows) == ROWS_683, len(rows)
    for row in export['files']:
        assert mapping[row['path']]['sha256'] == row['sha256'] and mapping[row['path']]['bytes'] == row['bytes'], row['path']
    for row in export['reuse_from_prior_imports']:
        assert mapping[row['path']]['sha256'] == row['sha256'], row['path']
    local = next(iter(export['chapters'].values()))['receipt'].split(REL)[0]
    assert local.startswith('/') and local.endswith('/'), local
    # The export also contains pinned Cargo/engine support outside the video repo.
    # Only chapter metadata and the table paths to be rebased must share its root.
    for entry in export['chapters'].values():
        for key in ('receipt', 'summary', 'inputs'):
            assert entry[key].startswith(local) and entry[key] in mapping, (key, entry[key])
    return dict(receipt=receipt, result=result, manifest=manifest, export=export, mapping=mapping, local=local)


def physical(state, path):
    row = state['mapping'][path]
    found = Path(row['physical'])
    assert found.is_file() and not found.is_symlink(), path
    return found


def rebase_path(value, local):
    assert value.startswith(local), value
    return str(A/'repo')+'/'+value.removeprefix(local)


def rebase_table(text, name, local):
    """Path column only. Any other change, or a Mac path anywhere else, fails here or at the byte comparison."""
    out = []
    for line in text.split('\n'):
        fields = line.split('\t')
        if name.endswith('.cuelog.tsv'):
            if line and not line.startswith('#'):
                assert len(fields) == 13, line
                fields[0] = rebase_path(fields[0], local)
        elif fields[0] == 'line':
            assert len(fields) == 5, line
            fields[4] = rebase_path(fields[4], local)
        else:
            assert not line or line.startswith('#') or fields[0] in TIMELINE_KINDS, ('unknown timeline row', line)
        out.append('\t'.join(fields))
    result = '\n'.join(out)
    assert local not in result, name
    return result


def local_chapter(ch, state, comp, o):
    """A chapter finished on the Mac: its receipt, summary and files, resolved only through the import records."""
    entry = state['export']['chapters'][ch]
    receipt_path = physical(state, entry['receipt'])
    o.success(receipt_path)
    receipt = read(receipt_path)
    assert receipt['inputs_sha256'] == state['mapping'][entry['inputs']]['sha256'], ch
    summary = read(physical(state, entry['summary']))
    assert set(summary['chapters']) == {ch} and summary['single_chapter'] == ch, ch
    row = summary['chapters'][ch]
    folder = REL+'/apportionment-final632-'+ch+'/render'
    assert row['folder'] == folder and row['frames'] == entry['frames'] == comp['chapters'][ch]['frames'], ch
    files = {}
    for rel, digest in row['files'].items():
        assert not Path(rel).is_absolute() and '..' not in Path(rel).parts, rel
        key = state['local']+folder+'/'+rel
        assert state['mapping'][key]['sha256'] == digest, key
        files[rel] = physical(state, key)
    return dict(chapter=ch, imported=True, frames=row['frames'], settings=row['encoder_settings'], digests=row['files'],
                files=files, records=[receipt_path, physical(state, entry['summary']), physical(state, entry['inputs'])])


def csd_chapter(ch, comp, version, tags, o):
    """A chapter finished here by final672: the exact guarded command, its summary and every file at its recorded hash."""
    job = JOBS/('csd-apportionment-final672-'+ch)
    inputs = Q/('csd-apportionment-final672-'+ch+'-inputs.json')
    o.success(job/'resource-receipt.json')
    receipt = read(job/'resource-receipt.json')
    assert receipt['command'] == [PYTHON, '-I', '-B', str(Q/'apportionment-csd-final672.py'), ch], ch
    assert receipt['policy'] == POLICY and receipt['inputs_sha256'] == sha(inputs), ch
    summary = read(job/'final672-summary.json')
    assert summary['chapter'] == ch and summary['ffmpeg_version'] == version and summary['encoder_tags'] == list(tags), ch
    row = summary['result']
    assert row['folder'] == str((job/'render').relative_to(D)) and row['frames'] == comp['chapters'][ch]['frames'], ch
    files = {}
    for rel in row['files']:
        assert not Path(rel).is_absolute() and '..' not in Path(rel).parts, rel
        files[rel] = job/'render'/rel
        assert files[rel].is_file() and not files[rel].is_symlink(), rel
    return dict(chapter=ch, imported=False, frames=row['frames'], settings=row['encoder_settings'], digests=row['files'],
                files=files, records=[job/'resource-receipt.json', job/'final672-summary.json', inputs])


def check_tables(rec, comp, local, o):
    """Timeline666's tables are the reference for all 15 chapters. Nothing is written here."""
    ch, files, ref = rec['chapter'], rec['files'], T/'timeline'
    assert sha(ref/(ch+'.layout.tsv')) == comp['chapters'][ch]['layout_sha256'], ch
    for ext in ('timeline.tsv', 'cues.tsv', 'cuelog.tsv'):
        assert sha(ref/(ch+'.'+ext)) == comp['chapters'][ch]['tables'][ext], (ch, ext)
    same = ('cues.tsv', 'layout.tsv') if rec['imported'] else ('timeline.tsv', 'cues.tsv', 'layout.tsv')
    for ext in same:
        assert files[ch+'.'+ext].read_bytes() == (ref/(ch+'.'+ext)).read_bytes(), (ch, ext, 'differs from timeline666')
    if rec['imported']:
        name = ch+'.timeline.tsv'
        assert rebase_table(files[name].read_bytes().decode(), name, local).encode() == (ref/name).read_bytes(), (ch, 'timeline')
        name = ch+'.cuelog.tsv'
        lines = rebase_table(files[name].read_bytes().decode(), name, local).splitlines()
        want = o.stems(ch)
        stem = [x.split('\t') for x in lines if x.split('\t')[0] in want]
        assert [x for x in lines if x.split('\t')[0] not in want] == (ref/name).read_text().splitlines(), (ch, 'cue log')
        assert sorted(r[0] for r in stem) == sorted(want), (ch, 'stem rows')
        assert all((r[1], int(r[12])) == want[r[0]] for r in stem), (ch, 'stem hash or start sample')


def survey(o, state):
    """Every check that writes nothing, run by both modes. Returns the 15 chapter records in registry order."""
    for receipt in (A, BUILD, T):
        o.success(receipt/'resource-receipt.json')
    comp, qual = read(T/'comparison666.json'), read(BUILD/'qualification.json')
    assert comp['full_normalized_tables_equal'] is True and comp['total_frames'] == TOTAL_FRAMES and comp['score_qa_passed'] == 15
    assert sha(BINARY) == qual['binary_sha256']
    registry = qual['chapters']
    assert len(registry) == len(set(registry)) == 15 and set(registry) == set(comp['chapters'])
    assert set(state['export']['chapters']) == set(registry[:2]), 'the imported chapters are not 01 and 02'
    assert {p.stem for p in (STEMS/'qa').glob('*.json')} == set(registry)
    assert all(read(STEMS/'qa'/(c+'.json'))['passed'] is True for c in registry)
    script = 'explainer_corpus/apportionment/script.tsv'
    assert sha(A/'repo'/script) == state['mapping'][state['local']+script]['sha256'], 'the script differs from the exported one'
    version, tags = csd_encoder()
    records = [local_chapter(c, state, comp, o) for c in registry[:2]]
    records += [csd_chapter(c, comp, version, tags, o) for c in registry[2:]]
    assert sum(r['frames'] for r in records) == TOTAL_FRAMES
    for rec in records:
        check_tables(rec, comp, state['local'], o)
        assert sum(k.startswith('soundtrack/') for k in rec['files']) == 1, rec['chapter']
    mac, csd = [r['settings'] for r in records[:2]], [r['settings'] for r in records[2:]]
    assert len(set(mac)) == 1 and len(set(csd)) == 1, 'encoder settings differ within one host'
    assert {options(x) for x in mac+csd} == {options(o.X264)}, 'x264 options differ'
    return registry, comp, records, dict(mac_encoder_settings=mac[0], csd_encoder_settings=csd[0], mac_encoder_tags=list(MAC_TAGS),
                                         csd_encoder_tags=list(tags), csd_ffmpeg_version=version.splitlines()[0])


def freeze():
    assert not JOB.exists() and not INPUTS.exists(), 'no duplicate job or manifest'
    assert HELPER.parent == Q, HELPER
    o = load()
    state = imported(o)
    registry, comp, records, hosts = survey(o, state)
    timeline_manifest = Q/'csd-apportionment-timeline666-inputs.json'
    terminal = [state['receipt'], state['result'], A/'resource-receipt.json', BUILD/'resource-receipt.json', T/'resource-receipt.json']
    for rec in records[2:]:
        terminal += rec['records'][:2]
    stable(terminal)
    rows = {}

    def add(path, digest=None):
        path = Path(path)
        if str(path) in rows:                # 13 final672 manifests repeat one base; each file is hashed once
            assert digest is None or rows[str(path)] == digest, path
            return
        actual = sha(path)
        assert digest is None or actual == digest, path
        rows[str(path)] = actual

    add(timeline_manifest)
    for row in read(timeline_manifest)['files']:
        add(row['path'], row['sha256'])
    for row in state['mapping'].values():
        add(row['physical'], row['sha256'])
    for path in (HELPER, SUPPORT, D/'guard_csd02.py', Q/'apportionment-csd-final672.py', state['manifest'], state['result'],
                 state['receipt'], A/'resource-receipt.json', A/'relocation664.json', BUILD/'resource-receipt.json',
                 BUILD/'qualification.json', BINARY, T/'resource-receipt.json', T/'comparison666.json'):
        add(path)
    if CONTROLLER.exists():
        add(CONTROLLER)
    for table in sorted((T/'timeline').glob('*.tsv')):
        add(table)
    for rec in records:
        for path in rec['records']:
            add(path)
        for rel, path in rec['files'].items():
            add(path, rec['digests'][rel])
    for rec in records[2:]:
        for row in read(rec['records'][2])['files']:
            add(row['path'], row['sha256'])
    python, tools = Path(sys.executable).resolve(), [Path(FFMPEG).resolve(), Path(FFPROBE).resolve()]
    assert python == Path(PYTHON).resolve(), 'run with '+PYTHON
    for path in {python, *tools, *libraries(python), *libraries(tools[0]), *libraries(tools[1]), *libraries(BINARY)}:
        add(path)
    for rec in records:                      # every path the kit will open must be frozen before it runs
        ch = rec['chapter']
        for line in (T/'timeline'/(ch+'.timeline.tsv')).read_text().splitlines():
            if line.startswith('line\t'):
                assert line.split('\t')[4] in rows, line
        for stem in o.stems(ch):
            assert stem in rows, stem
    assert not JOB.exists() and not INPUTS.exists(), 'no duplicate job or manifest'
    with INPUTS.open('x') as f:
        json.dump(dict(files=[dict(path=k, sha256=v) for k, v in sorted(rows.items())]), f, indent=2)
    print('INPUTS700', len(rows), 'rows', sha(INPUTS), flush=True)


def guarded():
    """The exact guarded command in a fresh job folder under the frozen manifest and the unchanged resource policy."""
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


def recheck(bound):
    for path, digest in bound.items():
        assert sha(path) == digest, path


def materialize(records, local, out):
    """final/ as the kit reads it: hard links to the proven files, and for 01-02 path-rebased copies of two tables."""
    links = []
    for rec in records:
        ch = rec['chapter']
        capture = next(k for k in rec['files'] if k.startswith('soundtrack/'))
        for rel, source in rec['files'].items():
            digest = rec['digests'][rel]
            dest = out/('soundtrack/'+ch+'/soundtrack_0.wav' if rel == capture else rel)
            dest.parent.mkdir(parents=True, exist_ok=True)
            assert not dest.exists(), dest
            if rec['imported'] and rel in (ch+'.timeline.tsv', ch+'.cuelog.tsv'):
                with dest.open('xb') as f:
                    f.write(rebase_table(source.read_bytes().decode(), rel, local).encode())
                links.append(dict(source=str(source), source_sha256=digest, destination=str(dest), sha256=sha(dest),
                                  rebased_path_column_only=True))
            else:
                os.link(source, dest)
                assert sha(dest) == digest, dest
                links.append(dict(source=str(source), destination=str(dest), sha256=digest, rebased_path_column_only=False))
    return links


def recheck_imported_here(o, records, comp, out):
    """Support671's full chapter() on the materialized 01-02: this host's ffprobe and -xerror decode, the Mac encoder tags."""
    assert tuple(o.ENCODERS) == MAC_TAGS
    results = {}
    for rec in records[:2]:
        ch = rec['chapter']
        got = o.chapter(out, ch)
        capture = next(k for k in rec['files'] if k.startswith('soundtrack/'))
        assert got['frames'] == rec['frames'] and got['encoder_settings'] == rec['settings'], ch
        assert got['files'][ch+'.mp4'] == rec['digests'][ch+'.mp4'], ch
        assert got['files'][capture] == rec['digests'][capture], ch
        tables = comp['chapters'][ch]['tables']
        assert got['files'][ch+'.timeline.tsv'] == tables['timeline.tsv'] and got['files'][ch+'.cues.tsv'] == tables['cues.tsv'], ch
        assert got['files'][ch+'.layout.tsv'] == comp['chapters'][ch]['layout_sha256'], ch
        results[ch] = got
        print('IMPORTED_CHAPTER_CHECKED_HERE', ch, flush=True)
    return results


def run():
    assert HELPER.parent == Q, HELPER
    o = load()
    o.own_group()
    o.STATE['job'] = JOB
    for sig in o.STOP_SIGNALS:
        signal.signal(sig, o.on_stop)
    bound = guarded()
    recheck(bound)
    os.sched_setaffinity(0, {0, 1, 2, 3})
    os.environ.update(PATH='/usr/bin:/bin', RAYON_NUM_THREADS='4')
    state = imported(o)
    registry, comp, records, hosts = survey(o, state)
    chapters = subprocess.check_output([str(BINARY), 'chapters'], text=True, timeout=120).splitlines()
    assert chapters == registry, chapters
    for rec in records:
        for rel, path in rec['files'].items():
            assert bound.get(str(path)) == rec['digests'][rel], path
    out = JOB/'final'
    out.mkdir()
    links = materialize(records, state['local'], out)
    checked = recheck_imported_here(o, records, comp, out)
    with (JOB/'materialized700.json').open('x') as f:
        json.dump(dict(links=links, frames=TOTAL_FRAMES, hosts=hosts, imported_chapter_checks=checked,
                       chapters={r['chapter']: dict(frames=r['frames'], imported=r['imported'], encoder_settings=r['settings'])
                                 for r in records}), f, indent=2)
    tracked = {}
    for mode, command in (('master', [str(BINARY), 'master', '--narration', str(NARRATION), '--score', str(STEMS), '--out', str(out)]),
                          ('captions', [str(BINARY), 'captions', 'all', '--narration', str(NARRATION), '--out', str(out)])):
        code, tracked[mode] = o.run_owned(mode, command, 14400)
        assert code == 0, (mode, code)
        print('NATIVE_COMMAND_COMPLETED', mode, tracked[mode], flush=True)
    replays = [line.split()[0] for line in (JOB/'command.log').read_text(errors='replace').splitlines()
               if "replay = fmn's soundtrack, bit for bit" in line]
    assert replays == registry, ('the bit-for-bit replay lines do not name all chapters in order', replays)
    recheck(bound)
    movie, srt, vtt = out/'apportionment_master.mp4', out/'apportionment.en.srt', out/'apportionment.en.vtt'
    with (JOB/'master700-summary.json').open('x') as f:
        json.dump(dict(frames=TOTAL_FRAMES, chapters=registry, imported_chapters=registry[:2], movie=str(movie),
                       movie_sha256=sha(movie), srt_sha256=sha(srt), vtt_sha256=sha(vtt), hosts=hosts, tracking=tracked,
                       inputs_sha256=sha(INPUTS), helper_sha256=sha(HELPER), scope=SCOPE), f, indent=2)
    print('MASTER700_COMPLETED; verify701 and actual review pending', flush=True)


if __name__ == '__main__':
    assert len(sys.argv) == 2 and sys.argv[1] in ('inputs', 'run'), 'usage: apportionment-csd-master700.py inputs|run'
    (freeze if sys.argv[1] == 'inputs' else run)()
