"""Guarded CSD native master for apportionment: master700 with the repaired chapter 06 and the caption 168 repair.

Successor of master700. Prepared for root; its author never executed it. On the CSD only, with /usr/bin/python3 -I -B:
  inputs   no job: requires everything master700 required, with chapter 06 taken from root's reviewed manifest, repeats
           every check that writes nothing, and writes one new guard manifest.
  run      only as the child of guard_csd02: rechecks everything, builds final/ inside the new job folder, runs the
           native master and captions commands once, then repairs caption 168 before any summary hash.

What differs from master700, and why:
  1. Chapter 06 comes from the successor final job that CHAPTER06_MANIFEST names. That manifest is root's actual reviewed
     record after 731 and the full timing comparison; while it is unset both modes refuse, and this file names no
     chapter 06 job, script or receipt of its own. The job gets every check final672 jobs get here. Its timeline and cue
     tables must still equal timeline666's byte for byte (same timing); only its layout table, which the new picture
     changes, is held to the manifest's hash instead of timeline666's.
  2. After `captions all`, the reviewed 727 helper (pinned) moves caption 168's end from 616290 ms to 616310 ms. The
     kit's pair must equal master700's actual pair and the result must equal root's reviewed candidate (record 734). The
     kit's pair moves unchanged to captions-generated735/, the helper writes captions-repaired735/, and final/ then holds
     hard links to the repaired pair under the names the verifier reads. The summary keeps both pairs' hashes.
     This is a display dwell adjustment only: no speech, timing table, narration or verifier threshold changes.

Everything else is master700's code and guards, unchanged: chapters 01-02 through import683, the other twelve from
final672, build656's native binary for the chapters, master and captions commands, the resource policy, the four-CPU
affinity and four Rayon threads, and the bit-for-bit replay requirement.

Every recording stays the exact source file. No check is relaxed for the host: the kit's replay must still equal each
captured soundtrack bit for bit, so if Mac and CSD mixers differ this job stops, and that is the correct result.

This accepts no picture, motion, audio, fine-sync, AV or release result. A verify701 successor is a separate later job.
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
JOB = JOBS/'csd-apportionment-master770'
INPUTS = Q/'root-apportionment770-inputs.json'
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
SCOPE = ('Native master replay, frame grid and caption commands passed, and caption 168 was repaired for display dwell '
         'only. Chapters 01-02 were encoded on the Mac and 03-15 on the CSD, chapter 06 by its reviewed successor job. '
         'Separate final decode, caption, loudness, picture, motion, audio, fine-sync and independent AV checks remain; '
         'nothing here accepts any of them.')
# Root sets both from the actual reviewed chapter 06 manifest (a JSON in Q) after 731 and the full timing comparison.
CHAPTER06_MANIFEST = Q/'root-apportionment770-chapter06.json'
CHAPTER06_MANIFEST_SHA256 = '0c839a9670f3c991753c5ca9b5e441eecc564b043cfea25522bfd41ec7a588e5'
REPAIRED_NARRATION = JOBS/'csd-apportionment-ch06-prepare744/narration'
PICTURE_REVIEW = Q/'root-apportionment769-pictures-reviewed.json'
PICTURE_REVIEW_SHA = 'f76a104bb8dc90824042122647884fae0cbb3f1181a4a6d387cc97d0c3f2d4cf'
CHAPTER06_KEYS = ['chapter', 'inputs', 'job', 'layout_sha256', 'script', 'summary']
CAPTION_HELPER = Q/'apportionment-caption-repair727'/'repair_caption168.py'
CAPTION_HELPER_SHA = 'd94aab217a6ee6639328414b774a7a374fb610c0716f3213222ca421a3762b9f'
CAPTION_NAMES = ('apportionment.en.srt', 'apportionment.en.vtt')
# Root's record 734 (root-caption-actual734.json, sha256 3659f8fa12f535e5adbbe37d69f23110e27f9df6aeff66ec7a8bd395aee83324):
# this helper on master700's actual pair, before and after.
GENERATED_CAPTIONS = ('0568f2dd0f59cb2a207ba94426bb712f48a693a6bf2705d0dfb4c47a2b4aee9b',
                      '482c5817d52926f937e805cc3411904833a00878010559d216b507af85143690')
REPAIRED_CAPTIONS = ('61e636aea49ac7b8c0ab97f4a201a11932aa41e5c71d0bf66974bd7c236d96df',
                     '214e15cb76ad795adbd067ea89da52a3a344881d872bb058e7d8950268b5ef28')
CAPTION_SCOPE = ('Caption 168 ends at 616310 ms instead of 616290 ms: 84 characters over 4200 ms, at the unchanged limit of 20 '
                 'per second. Display dwell only, not evidence of speech timing; no speech, timing table or narration changed. '
                 "The kit's generated pair is kept unchanged in captions-generated735/.")


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


def require_chapter06():
    assert CHAPTER06_MANIFEST is not None and CHAPTER06_MANIFEST_SHA256 is not None, \
        'chapter 06 is not bound: root sets CHAPTER06_MANIFEST and its sha256 from the actual reviewed manifest'


def chapter06_binding(registry):
    """Root's reviewed manifest for the repaired chapter 06: its final job, guarded script, inputs, summary and layout."""
    require_chapter06()
    path = Path(CHAPTER06_MANIFEST)
    assert path.parent == Q and sha(path) == CHAPTER06_MANIFEST_SHA256, 'the chapter 06 manifest changed'
    b = read(path)
    assert sorted(b) == CHAPTER06_KEYS, sorted(b)
    ch = registry[5]
    assert ch.startswith('06_') and b['chapter'] == ch, (ch, b['chapter'])
    job, script, inputs = Path(b['job']), Path(b['script']), Path(b['inputs'])
    assert job.parent == JOBS and job.name != 'csd-apportionment-final672-'+ch, ('not a successor job', job)
    assert script.parent == Q and inputs.parent == Q, (script, inputs)
    assert re.fullmatch(r'[A-Za-z0-9_.-]+\.json', b['summary']) and re.fullmatch('[0-9a-f]{64}', b['layout_sha256']), b
    return dict(manifest=path, chapter=ch, job=job, script=script, inputs=inputs, summary=b['summary'],
                layout_sha256=b['layout_sha256'])


def successor_chapter(b, comp, version, tags, o):
    """The repaired chapter 06: csd_chapter's checks, with the names taken from the reviewed manifest."""
    ch, job, inputs = b['chapter'], b['job'], b['inputs']
    o.success(job/'resource-receipt.json')
    receipt = read(job/'resource-receipt.json')
    assert receipt['command'] == [PYTHON, '-I', '-B', str(b['script']), 'run', 'final',
        '--job', 'csd-apportionment-ch06-final757', '--build', 'csd-apportionment-ch06-build748',
        '--prepare', 'csd-apportionment-ch06-prepare744', '--timeline', 'csd-apportionment-ch06-timeline744'], ch
    assert sha(PICTURE_REVIEW) == PICTURE_REVIEW_SHA
    review = read(PICTURE_REVIEW)
    assert review['sheets_viewed'] == list(range(1, 40)) and review['sample_count'] == 310
    assert receipt['policy'] == POLICY and receipt['inputs_sha256'] == sha(inputs), ch
    summary = read(job/b['summary'])
    assert summary['chapter'] == ch and summary['ffmpeg_version'] == version and summary['encoder_tags'] == list(tags), ch
    row = summary['result']
    assert row['folder'] == str((job/'render').relative_to(D)) and row['frames'] == comp['chapters'][ch]['frames'], ch
    files = {}
    for rel in row['files']:
        assert not Path(rel).is_absolute() and '..' not in Path(rel).parts, rel
        files[rel] = job/'render'/rel
        assert files[rel].is_file() and not files[rel].is_symlink(), rel
    return dict(chapter=ch, imported=False, frames=row['frames'], settings=row['encoder_settings'], digests=row['files'],
                files=files, records=[job/'resource-receipt.json', job/b['summary'], inputs, b['manifest'], PICTURE_REVIEW],
                layout_sha256=b['layout_sha256'])


def repaired_table(text, name):
    """Rebind only known narration path columns after proving each full WAV identical."""
    out = []
    count = 0
    for line in text.split('\n'):
        fields = line.split('\t')
        index = 4 if fields[0] == 'line' and name.endswith('.timeline.tsv') else (
            0 if name.endswith('.cuelog.tsv') and fields[0].startswith(str(REPAIRED_NARRATION)+'/') else None)
        if index is not None:
            source = Path(fields[index])
            assert source.parent == REPAIRED_NARRATION and source.suffix == '.wav', source
            destination = NARRATION/source.name
            assert sha(source) == sha(destination), ('full WAV differs', source)
            fields[index] = str(destination)
            count += 1
        out.append('\t'.join(fields))
    assert count == 6, (name, count)
    result = '\n'.join(out)
    assert str(REPAIRED_NARRATION) not in result
    return result


def check_tables(rec, comp, local, o):
    """Timeline666's tables are the reference for all 15 chapters. Nothing is written here."""
    ch, files, ref = rec['chapter'], rec['files'], T/'timeline'
    assert sha(ref/(ch+'.layout.tsv')) == comp['chapters'][ch]['layout_sha256'], ch
    for ext in ('timeline.tsv', 'cues.tsv', 'cuelog.tsv'):
        assert sha(ref/(ch+'.'+ext)) == comp['chapters'][ch]['tables'][ext], (ch, ext)
    same = ('cues.tsv', 'layout.tsv') if rec['imported'] else ('timeline.tsv', 'cues.tsv', 'layout.tsv')
    if 'layout_sha256' in rec:                   # the repaired chapter 06: same timing, new picture
        same = ('cues.tsv',)
        assert sha(files[ch+'.layout.tsv']) == rec['layout_sha256'], (ch, 'layout differs from the reviewed manifest')
        name = ch+'.timeline.tsv'
        assert repaired_table(files[name].read_text(), name) == (ref/name).read_text(), (ch, 'repaired timeline')
        name = ch+'.cuelog.tsv'
        lines = repaired_table(files[name].read_text(), name).splitlines()
        want = o.stems(ch)
        stems = [x.split('\t') for x in lines if x.split('\t')[0] in want]
        assert [x for x in lines if x.split('\t')[0] not in want] == (ref/name).read_text().splitlines()
        assert sorted(r[0] for r in stems) == sorted(want)
        assert all((r[1], int(r[12])) == want[r[0]] for r in stems)
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
    binding = chapter06_binding(registry)
    records = [local_chapter(c, state, comp, o) for c in registry[:2]]
    records += [successor_chapter(binding, comp, version, tags, o) if c == binding['chapter'] else
                csd_chapter(c, comp, version, tags, o) for c in registry[2:]]
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
    require_chapter06()
    caption_helper()
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
    for path in (HELPER, SUPPORT, CAPTION_HELPER, D/'guard_csd02.py', Q/'apportionment-csd-final672.py', state['manifest'], state['result'],
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
    print('INPUTS735', len(rows), 'rows', sha(INPUTS), flush=True)


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
            if 'layout_sha256' in rec and rel in (ch+'.timeline.tsv', ch+'.cuelog.tsv'):
                with dest.open('xb') as f:
                    f.write(repaired_table(source.read_text(), rel).encode())
                links.append(dict(source=str(source), source_sha256=digest, destination=str(dest), sha256=sha(dest),
                                  repaired_narration_path_column_only=True))
            elif rec['imported'] and rel in (ch+'.timeline.tsv', ch+'.cuelog.tsv'):
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


def caption_helper():
    """The reviewed 727 helper by its pin, loaded the way support671 is."""
    assert sha(CAPTION_HELPER) == CAPTION_HELPER_SHA, 'caption helper 727 changed'
    spec = importlib.util.spec_from_file_location('repair727', CAPTION_HELPER)
    h = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(h)
    values = (h.CUE, h.START_MS, h.OLD_END_MS, h.NEW_END_MS, h.NEXT_START_MS, h.LIMIT, h.SRT_NAME, h.VTT_NAME)
    assert values == (168, 612110, 616290, 616310, 616900, 20, *CAPTION_NAMES), ('caption helper 727 values changed', values)
    return h


def repair_captions(out, job):
    """727 on the pair `captions all` just wrote, before any summary hash. Nothing is rewritten in place: the kit's pair
    moves unchanged to captions-generated735/, the helper writes captions-repaired735/, and final/ gets hard links to it."""
    h = caption_helper()
    generated, repaired = job/'captions-generated735', job/'captions-repaired735'
    assert not generated.exists() and not repaired.exists(), 'caption folders already exist'
    assert tuple(sha(out/n) for n in CAPTION_NAMES) == GENERATED_CAPTIONS, 'the kit wrote another caption pair than master700'
    report = h.run(out/CAPTION_NAMES[0], out/CAPTION_NAMES[1], *GENERATED_CAPTIONS, repaired)
    assert (report['output_srt_sha256'], report['output_vtt_sha256']) == REPAIRED_CAPTIONS, report
    assert tuple(sha(repaired/n) for n in CAPTION_NAMES) == REPAIRED_CAPTIONS
    generated.mkdir()
    for name in CAPTION_NAMES:
        assert not (generated/name).exists() and not (out/name).is_symlink(), name
        os.rename(out/name, generated/name)
        os.link(repaired/name, out/name)
    assert tuple(sha(generated/n) for n in CAPTION_NAMES) == GENERATED_CAPTIONS
    assert tuple(sha(out/n) for n in CAPTION_NAMES) == REPAIRED_CAPTIONS
    print('CAPTION168_REPAIRED display dwell only', flush=True)
    return dict(helper=str(CAPTION_HELPER), helper_sha256=CAPTION_HELPER_SHA, reviewed_record='root-caption-actual734.json',
                generated=dict(folder=str(generated), srt_sha256=GENERATED_CAPTIONS[0], vtt_sha256=GENERATED_CAPTIONS[1]),
                repaired=dict(folder=str(repaired), srt_sha256=REPAIRED_CAPTIONS[0], vtt_sha256=REPAIRED_CAPTIONS[1]),
                final_hard_links=[str(out/n) for n in CAPTION_NAMES], report=report, scope=CAPTION_SCOPE)


def finish(registry, hosts, tracked, out):
    """The tail of run: the caption repair, then the summary, whose hashes are of what final/ now holds."""
    captions = repair_captions(out, JOB)
    movie, srt, vtt = out/'apportionment_master.mp4', out/'apportionment.en.srt', out/'apportionment.en.vtt'
    with (JOB/'master735-summary.json').open('x') as f:
        json.dump(dict(frames=TOTAL_FRAMES, chapters=registry, imported_chapters=registry[:2], movie=str(movie),
                       movie_sha256=sha(movie), srt_sha256=sha(srt), vtt_sha256=sha(vtt), caption_repair=captions,
                       chapter06_manifest=str(CHAPTER06_MANIFEST), chapter06_manifest_sha256=CHAPTER06_MANIFEST_SHA256,
                       hosts=hosts, tracking=tracked, inputs_sha256=sha(INPUTS), helper_sha256=sha(HELPER), scope=SCOPE),
                  f, indent=2)


def run():
    assert HELPER.parent == Q, HELPER
    require_chapter06()
    o = load()
    o.own_group()
    o.STATE['job'] = JOB
    for sig in o.STOP_SIGNALS:
        signal.signal(sig, o.on_stop)
    bound = guarded()
    recheck(bound)
    caption_helper()
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
    with (JOB/'materialized735.json').open('x') as f:
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
    finish(registry, hosts, tracked, out)
    print('MASTER735_COMPLETED; a verify701 successor and actual review pending', flush=True)


if __name__ == '__main__':
    assert len(sys.argv) == 2 and sys.argv[1] in ('inputs', 'run'), 'usage: apportionment-csd-master735.py inputs|run'
    (freeze if sys.argv[1] == 'inputs' else run)()
