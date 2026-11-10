"""Final 757 (CC0): the repaired chapter 06 as one 4K final on the CSD, then its selected pictures. Two guarded jobs.

Prepared only: it has not run on any host. It renders chapter 06 and nothing else. It assembles no master, sets no
master slot and accepts no picture, motion, listening, fine-sync, AV or release result. No audio heard or checked.

Gate, checked again at the start of every phase:
  build748    finished, stable, run by its sealed helper, its workspace at INSTALLED744 with the exact 722 sources
              (prepare744's own checks, called from that pinned file).
  prepare744  finished, stable, rebind744.json as written, its narration folder unchanged.
  timeline744 finished with exit 0 and stable inputs, run by prepare744.py with exactly these job names, no
              mismatch744.json, and comparison744.json showing 45 of 45 normalized tables equal, 100,886 frames and
              every chapter's frame count equal to 666's. Chapter 06's four tables at the hashes that record gives.
              The 15 score QA files passed, at the hashes timeline744's manifest bound. The 06 score stem is at the
              hash timeline666's manifest bound (667 bound every file under 664's repo).

Phase `final` (job csd-apportionment-ch06-final757): final672's command for one chapter, with apportionment-native748
and prepare744's narration: 3840x2160, 60 fps, --lint, the score445 stems, CRF 16, preset slow, 4 threads, CPUs 0-3,
RAYON_NUM_THREADS=4, PATH=/usr/bin:/bin, the 14,400 s deadline. support671's run_owned() and chapter() run it and check
it, as final672 does, with one change of reference: the timeline, cues and layout must be byte-equal to timeline744's
06 tables, and the cue log must be timeline744's plus exactly the 06 stem rows. Encoder identity: this host's ffmpeg
version and encoder tags, and the x264 settings string, must each equal the one value that the 12 retained CSD
final672 chapters and the old 06 final all recorded. A difference among those 13 refuses at `inputs`.
Writes final757-summary.json.

Phase `pictures` (job csd-apportionment-ch06-pictures757, after final757): pictures678's extraction. The same
import683, seed plan 677 and support616. Every seed sample of 06 and its reasons are kept. These include the original
prediction anchors: mark 365, +0.3 s, +2.0 s, the frame before 06_2, 06_2's start and +0.5 s. Every row of the new
final's layout.tsv gets its first look, persistence and grid samples. Windows for repair 722's changes are added,
counted from the final's own cue starts:
  06_1 inputs (wedges, Cedar's wedge at the top tick, step labels, question): every 0.1 s through their 0.6 s fade-in,
       every 0.5 s through the 06_1 hold, every 2 frames through the 0.2 s "?" fade at 06_2's start;
  06_3 arrival tags and summary line: every 0.2 s through 06_3 and its hold;
  06_4 year line: every 0.2 s through 06_4 and its hold;
  persistence: every 0.5 s from 06_5's start to the last frame, because nothing in the scene removes the year line.
Writes the frames, 8-picture sheets, sampling.json and pictures757-summary.json.

The 14 retained chapters are checked as found and recorded, never changed: 01 and 02 by export 682's pinned hashes
through import683's physical files, and 03-05 and 07-15 as their final672 jobs recorded them. The old 06 final is
checked and kept. Reusing any of them, choosing the new 06 and building a master are root's decisions.

Usage, under guard_csd02 (see handoff758); every job name may take -aN:
  apportionment-ch06-final757.py inputs|run final --job JOB --build B748 --prepare P744 --timeline T744
  apportionment-ch06-final757.py inputs|run pictures --job JOB --build B748 --prepare P744 --timeline T744 --final F757
`inputs` checks everything it can without the guard and writes only the guard's manifest. `run` is the guard's command.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import platform
import re
import signal
import subprocess
import sys
import time

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
JOBS = D/'jobs'
PYTHON = '/usr/bin/python3'
FFMPEG = '/usr/bin/ffmpeg'
FFPROBE = '/usr/bin/ffprobe'
GUARD = D/'guard_csd02.py'
STOPPER = D/'qualification01/owned-process-stop02.py'
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4, work_cap_gib=56, target_growth_cap_gib=20,
              sample_seconds=10)
JOB_NAMES = dict(final=r'csd-apportionment-ch06-final757(-a[0-9]+)?', pictures=r'csd-apportionment-ch06-pictures757(-a[0-9]+)?',
                 build=r'csd-apportionment-ch06-build748(-a[0-9]+)?', prepare=r'csd-apportionment-ch06-prepare744(-a[0-9]+)?',
                 timeline=r'csd-apportionment-ch06-timeline744(-a[0-9]+)?')
HELPER = Path(__file__).resolve()
# Pinned helpers this one loads; nothing in them runs at import.
PREPARE744 = (Q/'apportionment-ch06-prepare744.py', '1fe9efc589fe575ece6a1fe90a31cf6a5156fd52ad2c8db931ad30466a6b0867')
SUPPORT671 = (Q/'apportionment-csd-support671.py', '244fe1a47d5a40f71594031bafc09fefb45558c38ed72adbfb6aa1b1f269c40a')
PICTURES678 = (Q/'apportionment-csd-pictures678.py', '7a2942b6bd33765e40076294d88473505b531b900a2f097aacd3d95cf0ffa061')
FINAL672 = (Q/'apportionment-csd-final672.py', '54262bf86dda15e86d08c23d90f19d58d7d0c0e5c61c466720072646be544f24')
REL = 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
STEMS = JOBS/'csd-apportionment-assembly664/repo'/REL/'apportionment-frederick-score445/score/stems'
IMPORT683 = JOBS/'csd-apportionment-import683-a1'
EXPORT682_SHA = '1c7c690b603ed0a3571ba27e033dc2c1af581cacf74b171e01996e702e2285c8'
CHAPTER = '06_not_every_time'
FRAMES06 = 4947
RETAINED_CSD = ('03_largest_tails', '04_sixteenth', '05_dials', '07_yardstick', '08_signposts', '09_cost',
                '10_rounding_rules', '11_webster_quota', '12_both_promises', '13_population', '14_no_rule', '15_recap')
# Chapters 01 and 02: the local final632 files as export 682 lists them (path under REL: sha256), and their frames.
LOCAL632 = {
    'apportionment-final632-01_one_more_seat/final632-summary.json': 'fa3bab6a1505279da9f6f53f2216db35e54f709b6941c02f1f2ce87def51ec64',
    'apportionment-final632-01_one_more_seat/render/01_one_more_seat.cuelog.tsv': 'f993d82af405c3e2fcd5cf94d174b5edc6fe526a48f6d4941576e2e055caa67a',
    'apportionment-final632-01_one_more_seat/render/01_one_more_seat.cues.tsv': 'ad408c17122d51115c33f117b63a5f44e35c979554026131c36d05fb5d1ee393',
    'apportionment-final632-01_one_more_seat/render/01_one_more_seat.layout.tsv': 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    'apportionment-final632-01_one_more_seat/render/01_one_more_seat.mp4': 'bc3725c2722d6cd31d2918cf94ab7a87395ad98cac83ab2bbbbb39546708a174',
    'apportionment-final632-01_one_more_seat/render/01_one_more_seat.timeline.tsv': '28a41228dc55e9ce13202ae2af2adac208e9819ba1d395490f43fba6b11612a8',
    'apportionment-final632-01_one_more_seat/render/soundtrack/01_one_more_seat/soundtrack_0.wav': '64bf575b2bce8ad26e27c599a6d2a4b39d08c4219a8251dd1fb30bdd306a3293',
    'apportionment-final632-01_one_more_seat/resource-receipt.json': '7744dc98c140893055c44101270b0d127c2ce6a8474c64cddc9fba18ea0c72ae',
    'apportionment-final632-02_shares/final632-summary.json': '3c1891f2a1056932f8616fd63cfea53fc059c363dce58e5ede2572741dfb4aff',
    'apportionment-final632-02_shares/render/02_shares.cuelog.tsv': '2e5cd23b0fe80a2e359e8e1fa2116cffff3f087053bd9ba0ab8438b61b384715',
    'apportionment-final632-02_shares/render/02_shares.cues.tsv': '22f502d908f3a70f66e53f5c0a18d2aa7159af0e955c74cbbf21c17eec672c98',
    'apportionment-final632-02_shares/render/02_shares.layout.tsv': 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    'apportionment-final632-02_shares/render/02_shares.mp4': 'cdc9067532923b2ef466473a5442ec2f30f6687ae29a23ff87e2e8d3680bcbd9',
    'apportionment-final632-02_shares/render/02_shares.timeline.tsv': '205d5e8c996afe80d048117a32824df90320f77fc8146926adb661d34aa92296',
    'apportionment-final632-02_shares/render/soundtrack/02_shares/soundtrack_0.wav': 'ddeb1646247de8114c12f6bac0789e98b10319e7336c38a622804676b707031c',
    'apportionment-final632-02_shares/resource-receipt.json': '479594d6d0e11dd45b168ca6244c466eaa54b0008657ff7ca8cc6c4c1113129c',
}
LOCAL632_FRAMES = {'01_one_more_seat': 4806, '02_shares': 6647}
RENDER_SECONDS = 14400                                 # final672's deadline for one chapter
STABLE_SECONDS = 15                                    # support616's two reads of a finished receipt
NARRATION = 'apportionment-picture-assembly566/narration'
TABLES567 = REL + '/apportionment-picture-timeline567/timeline/'
SCOPE_FINAL = ('The repaired chapter 06 as one 4K final, with final672\'s technical checks against timeline744. No master is '
               'assembled and no master slot is set. No picture, motion, listening, fine-sync, cross-host master, AV or '
               'release acceptance. The 14 retained chapters are recorded as found; reusing them is root\'s decision.')
SCOPE_PICTURES = ('Exact selected frames from the new 4K chapter 06 final, not yet viewed. No picture, motion, sound, '
                  'fine-sync, master, AV or release acceptance.')


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8'))


def load(pinned, name):
    """A pinned helper, loaded from its CSD path after its hash is checked."""
    path, digest = pinned
    assert sha(path) == digest, ('pinned helper changed', str(path))
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def modules():
    m744, o = load(PREPARE744, 'prepare744'), load(SUPPORT671, 'owned671')
    o.R, o.STEMS, o.FFMPEG, o.FFPROBE = D, STEMS, FFMPEG, FFPROBE      # as final672 sets them; REF is set per run
    return m744, o


def stable(paths):
    """support616's rule: a finished record reads the same twice, STABLE_SECONDS apart."""
    first = {str(p): sha(p) for p in paths}
    time.sleep(STABLE_SECONDS)
    assert first == {str(p): sha(p) for p in paths}, 'a finished record changed between two reads'


def gate(m744, o, args):
    """Build 748, prepare744 and timeline744 as above. Returns (rows, binary, workspace, narration, comparison744)."""
    rows, q748, binary, workspace = m744.evidence748(args.build)
    assert m744.listing(workspace) == m744.INSTALLED744, '748 workspace is not as prepare744 installed it'
    assert m744.listing(workspace/m744.TOPIC) == m744.CRATE744, '748 crate is not as prepare744 installed it'
    for rel, digest in m744.tree_rows(workspace).items():
        rows[str(workspace/rel)] = digest
    more, narration = m744.check_rebind_record(args.prepare, workspace)
    rows.update(more)
    job = JOBS/args.timeline
    receipt, record = job/'resource-receipt.json', job/'comparison744.json'
    r = m744.success(job, '744 timeline')
    stable([receipt, record])
    assert r['command'] == [PYTHON, '-I', '-B', str(PREPARE744[0]), 'run', 'timeline', '--job', args.timeline, '--build',
                            args.build, '--prepare', args.prepare], ('744 timeline command', r['command'])
    manifest = Q/(args.timeline + '-inputs.json')
    assert sha(manifest) == r['inputs_sha256'], '744 timeline receipt binds another manifest'
    assert not (job/'mismatch744.json').exists(), 'timeline744 wrote a mismatch record'
    comp = read(record)
    assert comp['schema'] == 'apportionment-ch06-timeline744' and comp['build'] == args.build, '744 comparison'
    assert comp['prepare'] == args.prepare and comp['narration'] == str(narration), '744 comparison'
    assert comp['binary'] == str(binary) and comp['binary_sha256'] == sha(binary), '744 comparison binary'
    assert comp['rebind744_sha256'] == sha(JOBS/args.prepare/'rebind744.json'), '744 comparison rebind record'
    assert comp['inputs_sha256'] == r['inputs_sha256'], '744 comparison manifest'
    assert comp['full_normalized_tables_equal'] is True and comp['problems'] == [], ('744 problems', comp['problems'])
    assert comp['tables_compared'] == comp['tables_equal'] == 45, '744 tables'
    assert list(comp['chapters']) == m744.CHAPTERS and comp['total_frames'] == comp['total_frames_666'] == m744.FRAMES666
    for ch, x in comp['chapters'].items():
        assert x['frames'] == x['frames_666'] and all(x['normalized_equal_666'][e] is True for e in m744.TABLES), ch
    assert comp['chapters'][CHAPTER]['frames'] == FRAMES06, '06 frames'
    tables = dict(comp['chapters'][CHAPTER]['tables'], **{'layout.tsv': comp['chapters'][CHAPTER]['layout_sha256']})
    for ext, digest in tables.items():
        path = job/'timeline'/(CHAPTER + '.' + ext)
        assert sha(path) == digest, ('timeline744 06 table changed', ext)
        rows[str(path)] = digest
    bound744 = m744.manifest_rows(manifest)
    for ch in m744.CHAPTERS:
        path = STEMS/'qa'/(ch + '.json')
        assert read(path)['passed'] is True and bound744.get(str(path)) == sha(path), ('score QA', ch)
        rows[str(path)] = sha(path)
    assert sorted(p.stem for p in (STEMS/'qa').glob('*.json')) == m744.CHAPTERS, 'score QA files'
    more, bound666 = m744.evidence666()                 # 667 bound every file under 664's repo for timeline666
    rows.update(more)
    for path, (digest, _) in o.stems(CHAPTER).items():
        assert bound666.get(path) == digest, ('06 score stem is not the one timeline666 bound', path)
        rows[path] = digest
    for path in (receipt, record, manifest):
        rows[str(path)] = sha(path)
    return rows, binary, workspace, narration, comp


def final672(m744, c):
    """One successful final672 chapter job, as its summary records it. Returns (rows, summary, record)."""
    job = JOBS/('csd-apportionment-final672-' + c)
    receipt, summary_path = job/'resource-receipt.json', job/'final672-summary.json'
    r = m744.success(job, '672 ' + c)
    assert r['command'][-2:] == [str(FINAL672[0]), c], ('672 command', c)
    manifest = Q/('csd-apportionment-final672-' + c + '-inputs.json')
    assert sha(manifest) == r['inputs_sha256'], ('672 manifest', c)
    s = read(summary_path)
    assert s['chapter'] == c and s['result']['folder'] == str((job/'render').relative_to(D)), ('672 summary', c)
    rows = {str(p): sha(p) for p in (receipt, summary_path, manifest)}
    for name, digest in s['result']['files'].items():
        assert not Path(name).is_absolute() and '..' not in Path(name).parts, name
        path = job/'render'/name
        assert not path.is_symlink() and sha(path) == digest, ('672 file changed', c, name)
        rows[str(path)] = digest
    record = dict(source='CSD final672', job=job.name, frames=s['result']['frames'], receipt_sha256=rows[str(receipt)],
                  summary_sha256=rows[str(summary_path)], movie_sha256=s['result']['files'][c + '.mp4'])
    return rows, s, record


def retained(m744, comp):
    """The 14 retained chapters and the old 06, checked and recorded unchanged. Returns (rows, record, identity)."""
    rows = {str(FINAL672[0]): sha(FINAL672[0])}
    assert rows[str(FINAL672[0])] == FINAL672[1], 'final672 helper changed'
    record, identity = {}, dict(ffmpeg_version=set(), encoder_tags=set(), encoder_settings=set())
    for c in RETAINED_CSD + (CHAPTER,):
        more, s, rec = final672(m744, c)
        assert rec['frames'] == comp['chapters'][c]['frames'], ('672 frames', c)
        rows.update(more)
        identity['ffmpeg_version'].add(s['ffmpeg_version'])
        identity['encoder_tags'].add(tuple(s['encoder_tags']))
        identity['encoder_settings'].add(s['result']['encoder_settings'])
        record[c] = rec
    old06 = dict(record.pop(CHAPTER), note='The old 06 final, preserved; superseded only if root accepts the new 06.')
    assert all(len(v) == 1 for v in identity.values()), 'the 13 CSD final672 chapters do not share one encoder identity'
    identity = {k: next(iter(v)) for k, v in identity.items()}
    receipt, result_path = IMPORT683/'resource-receipt.json', IMPORT683/'import-result.json'
    m744.success(IMPORT683, 'import683')
    result = read(result_path)
    assert result['manifest_sha256'] == EXPORT682_SHA, 'import683 carried another manifest'
    imported = result['files'] + result['reused_files']
    rows.update({str(receipt): sha(receipt), str(result_path): sha(result_path)})
    files = {c: {} for c in LOCAL632_FRAMES}
    for rel, digest in LOCAL632.items():
        found = [x for x in imported if x['path'].endswith('/' + REL + '/' + rel)]
        assert len(found) == 1 and found[0]['sha256'] == digest, ('export 682 row', rel)
        physical = Path(found[0]['physical'])
        assert not physical.is_symlink() and sha(physical) == digest, ('imported file changed', rel)
        rows[str(physical)] = digest
        files[rel.split('/')[0].removeprefix('apportionment-final632-')][rel] = digest
    for c, frames in LOCAL632_FRAMES.items():
        assert comp['chapters'][c]['frames'] == frames, ('632 frames', c)
        record[c] = dict(source='local final632, export 682, imported by import683', frames=frames, files=files[c])
    assert sorted(record) == sorted(m744.CHAPTERS[:5] + m744.CHAPTERS[6:]), sorted(record)
    return rows, dict(sorted(record.items())), old06, identity


def host_rows(m744, *paths):
    rows = {str(HELPER): sha(HELPER), str(GUARD): sha(GUARD), str(STOPPER): sha(STOPPER)}
    for pinned in (PREPARE744, SUPPORT671) + paths:
        rows[str(pinned[0])] = sha(pinned[0])
    for tool in (PYTHON, FFMPEG, FFPROBE):
        rows[str(Path(tool).resolve())] = sha(Path(tool).resolve())
    rows.update(m744.guard_pins())
    return rows


def final757(m744, args):
    """Finished final757, as its summary records it. Returns (rows, summary)."""
    job = JOBS/args.final
    receipt, summary_path = job/'resource-receipt.json', job/'final757-summary.json'
    r = m744.success(job, '757 final')
    stable([receipt, summary_path])
    assert r['command'] == [PYTHON, '-I', '-B', str(HELPER), 'run', 'final', '--job', args.final, '--build', args.build,
                            '--prepare', args.prepare, '--timeline', args.timeline], ('757 final command', r['command'])
    manifest = Q/(args.final + '-inputs.json')
    assert sha(manifest) == r['inputs_sha256'], '757 final receipt binds another manifest'
    s = read(summary_path)
    assert s['schema'] == 'apportionment-ch06-final757' and s['chapter'] == CHAPTER and s['inputs_sha256'] == sha(manifest)
    assert (s['build'], s['prepare'], s['timeline']) == (args.build, args.prepare, args.timeline), '757 final jobs'
    assert s['result']['frames'] == FRAMES06 and s['result']['folder'] == str((job/'render').relative_to(D)), '757 final'
    rows = {str(p): sha(p) for p in (receipt, summary_path, manifest)}
    for name, digest in s['result']['files'].items():
        path = job/'render'/name
        assert not path.is_symlink() and sha(path) == digest, ('757 final file changed', name)
        rows[str(path)] = digest
    return rows, s


def bound_final(args, m744, o):
    rows, binary, workspace, narration, comp = gate(m744, o, args)
    more, record, old06, identity = retained(m744, comp)
    rows.update(more)
    rows.update(host_rows(m744))
    return rows, dict(m744=m744, o=o, binary=binary, workspace=workspace, narration=narration, comp=comp,
                      retained=record, old06=old06, identity=identity)


def windows(cues, frames):
    """Repair 722's changed elements, from the final's own cue starts. (frame, reason) pairs."""
    start = {cue: s for cue, s, _ in cues}
    assert sorted(start) == ['06_%d' % k for k in range(1, 7)], sorted(start)
    s1, s2, s3, s4, s5 = (start['06_%d' % k] for k in range(1, 6))
    out = []

    def every(lo, hi, step, reason):
        out.extend((f, reason) for f in range(lo, min(hi, frames), step))

    every(s1, s1 + 37, 6, '722 06_1 inputs fade in (0.6 s): every 0.1 s')
    every(s1 + 36, s2, 30, '722 06_1 wedges, Cedar at top tick, step labels, question: every 0.5 s')
    every(s2, s2 + 13, 2, '722 06_1 inputs leave in the 0.2 s "?" fade: every 2 frames')
    every(s3, s4, 12, '722 06_3 arrival tags and summary line: every 0.2 s')
    every(s4, s5, 12, '722 06_4 year line: every 0.2 s')
    every(s5, frames, 30, '722 year line persistence: every 0.5 s to the end')
    out.append((frames - 1, '722 persistence: chapter last frame'))
    return out


def merge757(h, seed, frames, cues, marks, layout):
    """pictures678's merge, labelled final757, plus the 722 windows. Every seed sample and reason is kept."""
    anchors = [dict(a) for a in seed['anchors']]
    known = {a['row']: a for a in anchors}
    final, new = h.read_layout(layout, 'final757'), []
    for a in final:
        if a['row'] not in known:
            known[a['row']] = a
            new.append(a)
        elif 'final757' not in known[a['row']]['sources']:
            known[a['row']]['sources'] = known[a['row']]['sources'] + ['final757']
    fresh = h.plan_chapter(CHAPTER, frames, cues, marks, anchors + new)
    assert [a['id'] for a in fresh['anchors'][:len(seed['anchors'])]] == [a['id'] for a in seed['anchors']], 'seed ids'
    assert fresh['predicts'] == seed['predicts'], ('prediction anchors moved', fresh['predicts'], seed['predicts'])
    picks = {s['frame']: dict(frame=s['frame'], seconds=s['seconds'], reasons=list(s['reasons'])) for s in seed['samples']}
    extra = [(s['frame'], r) for s in fresh['samples'] for r in s['reasons']] + windows(cues, frames)
    for frame, reason in extra:
        entry = picks.setdefault(frame, dict(frame=frame, seconds=round(frame/h.FPS, 4), reasons=[]))
        if reason not in entry['reasons']:
            entry['reasons'].append(reason)
    for s in seed['samples']:
        assert set(s['reasons']) <= set(picks[s['frame']]['reasons']), s['frame']
    samples = [picks[f] for f in sorted(picks)]
    assert samples and all(0 <= s['frame'] < frames for s in samples) and len(samples) <= h.MAX_SAMPLES
    prediction = [s['frame'] for s in samples if any(r.startswith('predict ') for r in s['reasons'])]
    changed = [s['frame'] for s in samples if any(r.startswith('722 ') for r in s['reasons'])]
    return dict(samples=samples, holds=fresh['holds'], predicts=fresh['predicts'], anchors=fresh['anchors'],
                skipped=seed['skipped'] + [x for x in fresh['skipped'] if x not in seed['skipped']],
                prediction_anchor_frames=prediction,
                merge=dict(seed_samples=len(seed['samples']), merged_samples=len(samples), final_layout_rows=len(final),
                           final_rows_not_in_seed=len(new), samples_722=len(changed)))


def bound_pictures(args, m744, o):
    p678 = load(PICTURES678, 'pictures678')
    assert p678.EXPORT_SHA == EXPORT682_SHA, '678 reads another export'
    rows, binary, workspace, narration, comp = gate(m744, o, args)
    more, summary = final757(m744, args)
    rows.update(more)
    assert summary['binary_sha256'] == sha(binary) and summary['narration'] == str(narration), '757 final inputs'
    s = p678.setup()
    h = s.h
    seed = s.plan['chapters'][CHAPTER]
    render = JOBS/args.final/'render'
    name, frames, lines, marks = h.read_timeline(render/(CHAPTER + '.timeline.tsv'))
    assert name == CHAPTER and frames == seed['frames'] == FRAMES06, (name, frames)
    for ext in ('timeline.tsv', 'cues.tsv'):
        made = (render/(CHAPTER + '.' + ext)).read_text().replace(str(narration), s.local + REL + '/' + NARRATION)
        assert made == s.bound[TABLES567 + CHAPTER + '.' + ext].read_text(), (ext, 'differs from bound timeline567')
    cues = h.read_cues(render/(CHAPTER + '.cues.tsv'), lines)
    movie = render/(CHAPTER + '.mp4')
    h.probe(movie, frames)
    plan = dict(chapter=CHAPTER, frames=frames, fps=h.FPS, registry_offset_frames=seed['registry_offset_frames'],
                **merge757(h, seed, frames, cues, marks, render/(CHAPTER + '.layout.tsv')))
    tools, provenance = p678.runtime(h)
    for path in set(s.files) | set(tools):
        rows[str(path)] = sha(path)
    rows.update(host_rows(m744, PICTURES678))
    plan.update(runtime=provenance, seed_plan_sha256=p678.PLAN_SHA,
                final=dict(job=args.final, summary_sha256=sha(JOBS/args.final/'final757-summary.json')))
    return rows, dict(m744=m744, o=o, h=h, plan=plan, movie=movie)


def read_receipt(job):
    for _ in range(10):                                # the guard rewrites it every 10 s while we run
        try:
            return json.loads((job/'resource-receipt.json').read_text())
        except json.JSONDecodeError:
            time.sleep(0.5)
    raise SystemExit('757 refused: the guard receipt could not be read')


def guard_binding(job, manifest):
    assert job.parent == JOBS and job.is_dir() and not job.is_symlink(), job
    present = sorted(p.name for p in job.iterdir())
    assert present == ['command.log', 'resource-receipt.json'], ('the job folder is not fresh', present)
    r = read_receipt(job)
    assert r['admitted'] is True and 'exit' not in r and 'finished' not in r, 'receipt state'
    assert r['phase'] == 'native' and r['preflight_only'] is False, 'receipt phase'
    assert r['policy'] == POLICY, ('policy', r['policy'])
    assert r['command'] == [PYTHON, '-I', '-B', str(HELPER)] + sys.argv[1:], ('command', r['command'])
    assert r['inputs_sha256'] == sha(manifest), 'the guard bound another manifest'
    return dict(job=job.name, command=r['command'], policy=r['policy'], admission=r.get('admission'),
                inputs_sha256=r['inputs_sha256'])


def check_host():
    assert sys.version_info >= (3, 11) and sys.flags.isolated and sys.dont_write_bytecode, 'run with python3 -I -B'
    assert platform.system() == 'Linux' and platform.machine() == 'x86_64', 'this helper runs on the CSD only'


def check_files(files):
    for row in files:
        assert sha(row['path']) == row['sha256'], ('input changed', row['path'])


def write_manifest(args, rows):
    manifest = Q/(args.job + '-inputs.json')
    files = [dict(path=path, sha256=rows[path]) for path in sorted(rows)]
    check_files(files)
    with manifest.open('x') as f:
        json.dump(dict(schema='apportionment-ch06-%s757-inputs' % args.phase, job=args.job, files=files), f, indent=1)
    print(json.dumps(dict(manifest=str(manifest), sha256=sha(manifest), files=len(files))))
    return 0


def bound(args, m744, o):
    return bound_final(args, m744, o) if args.phase == 'final' else bound_pictures(args, m744, o)


def start(args):
    """Common start under the guard: final672's process handling first, then the exact manifest."""
    m744, o = modules()
    o.own_group()
    job = JOBS/args.job
    o.STATE['job'] = job
    for sig in o.STOP_SIGNALS:
        signal.signal(sig, o.on_stop)
    manifest = Q/(args.job + '-inputs.json')
    guard = guard_binding(job, manifest)
    files = read(manifest)['files']
    check_files(files)
    rows, found = bound(args, m744, o)
    have = {row['path']: row['sha256'] for row in files}
    assert rows == have, ('the manifest does not bind exactly what this phase reads', sorted(set(rows) ^ set(have))[:20])
    return job, files, guard, found


def run_final(args):
    job, files, guard, x = start(args)
    m744, o, binary, comp, identity = x['m744'], x['o'], x['binary'], x['comp'], x['identity']
    o.REF = JOBS/args.timeline/'timeline'
    version = subprocess.check_output([FFMPEG, '-version'], text=True, timeout=30)

    def lib(name):
        found = re.search(r'^' + name + r'\s+(\d+)\.\s*(\d+)\.\s*(\d+)\s*/', version, re.M)
        assert found, name
        return '.'.join(found.groups())

    o.ENCODERS = ('Lavc' + lib('libavcodec') + ' libx264', 'Lavf' + lib('libavformat'))
    assert version == identity['ffmpeg_version'] and list(o.ENCODERS) == list(identity['encoder_tags']), \
        'this ffmpeg is not the one the retained CSD chapters used'
    os.sched_setaffinity(0, {0, 1, 2, 3})
    os.environ['RAYON_NUM_THREADS'] = '4'
    os.environ['PATH'] = '/usr/bin:/bin'               # python3 and ffmpeg as bound; the binary calls both by name
    chapters = subprocess.check_output([str(binary), 'chapters'], cwd=D, text=True, timeout=120).splitlines()
    assert chapters == m744.CHAPTERS, chapters
    out = job/'render'
    assert not out.exists()
    cmd = [str(binary), 'render', CHAPTER, '--res', '3840x2160', '--fps', '60', '--lint', '--narration', str(x['narration']),
           '--score', str(STEMS), '--out', str(out), '--crf', '16', '--preset', 'slow', '--threads', '4']
    print('FINAL757', CHAPTER, flush=True)
    started = time.monotonic()
    code, tracked = o.run_owned(CHAPTER, cmd, RENDER_SECONDS)
    assert code == 0, (CHAPTER, code)
    result = o.chapter(out, CHAPTER)
    assert result['frames'] == comp['chapters'][CHAPTER]['frames'] == FRAMES06, result['frames']
    assert result['encoder_settings'] == identity['encoder_settings'], 'x264 settings differ from the retained chapters'
    check_files(files)
    assert m744.listing(x['workspace']) == m744.INSTALLED744, '748 workspace changed during the render'
    with (job/'final757-summary.json').open('x') as f:
        json.dump(dict(
            schema='apportionment-ch06-final757', chapter=CHAPTER, node=platform.node(), guard=guard, build=args.build,
            prepare=args.prepare, timeline=args.timeline, binary=str(binary), binary_sha256=sha(binary),
            narration=str(x['narration']), report_sha256=sha(x['narration']/'report.json'),
            comparison744_sha256=sha(JOBS/args.timeline/'comparison744.json'), command=cmd, result=result,
            tracking=tracked, render_seconds=round(time.monotonic() - started, 1), ffmpeg_version=version,
            encoder_tags=o.ENCODERS, encoder_identity='ffmpeg version, encoder tags and x264 settings equal to the 12 '
            'retained CSD final672 chapters and the old 06', retained=x['retained'], old06=x['old06'],
            master=dict(assembled=False, slot_set=False, note='master735 keeps its chapter 06 slot unset; root decides.'),
            inputs_sha256=sha(Q/(args.job + '-inputs.json')), scope=SCOPE_FINAL), f, indent=2)
    print('TECHNICALLY_CHECKED', CHAPTER, flush=True)
    return 0


def run_pictures(args):
    job, files, guard, x = start(args)
    h, plan, movie = x['h'], x['plan'], x['movie']
    os.sched_setaffinity(0, {0, 1, 2, 3})
    folder = job/CHAPTER
    pictures = h.extract_frames(movie, [s['frame'] for s in plan['samples']], folder)
    assert len(pictures) == len(plan['samples'])
    for sample, path in zip(plan['samples'], pictures):
        sample.update(path=str(path), sha256=sha(path))
    plan.update(movie=dict(path=str(movie), sha256=sha(movie)), sheets=h.sheets(CHAPTER, plan['samples'], folder),
                guard=guard, scope=SCOPE_PICTURES)
    check_files(files)
    with (folder/'sampling.json').open('x') as f:
        json.dump(plan, f, indent=2)
        f.write('\n')
    done = dict(chapter=CHAPTER, frames=plan['frames'], samples=len(plan['samples']), sheets=len(plan['sheets']),
                prediction_anchor_frames=plan['prediction_anchor_frames'], merge=plan['merge'],
                sampling_sha256=sha(folder/'sampling.json'), movie_sha256=plan['movie']['sha256'],
                runtime=plan['runtime'], scope=SCOPE_PICTURES)
    with (job/'pictures757-summary.json').open('x') as f:
        json.dump(done, f, indent=2)
        f.write('\n')
    print(json.dumps(dict(done, runtime=None)), flush=True)
    return 0


def main():
    p = argparse.ArgumentParser(description=__doc__.split('\n', 1)[0])
    p.add_argument('mode', choices=('inputs', 'run'))
    p.add_argument('phase', choices=('final', 'pictures'))
    for name in ('job', 'build', 'prepare', 'timeline'):
        p.add_argument('--' + name, required=True)
    p.add_argument('--final')
    args = p.parse_args()
    for name in ('build', 'prepare', 'timeline'):
        assert re.fullmatch(JOB_NAMES[name], getattr(args, name)), '--%s is not a %s job name' % (name, name)
    assert re.fullmatch(JOB_NAMES[args.phase], args.job), 'job must be the %s757 name or that name plus -aN' % args.phase
    if args.phase == 'pictures':
        assert args.final and re.fullmatch(JOB_NAMES['final'], args.final), 'pictures needs --final'
    else:
        assert args.final is None, 'final takes no --final'
    check_host()
    if args.mode == 'inputs':
        assert not (JOBS/args.job).exists() and not (Q/(args.job + '-inputs.json')).exists(), 'no duplicate job'
        return write_manifest(args, bound(args, *modules())[0])
    return run_final(args) if args.phase == 'final' else run_pictures(args)


if __name__ == '__main__':
    raise SystemExit(main())
