"""CSD final technical verification and final picture extraction for the mixed complex master 318,
version 321. Prepared only; nothing here has run. It accepts no picture, motion, listening, fine-sync,
caption-reading or independent AV review; every flag it writes for those is False.

The master (cc0-combined-master318.py, CC0) mixes three binaries: 09_boundary rendered by binary 313
(final317), the ten other changed chapters by binary 268, and 02/03/13/15 retained from production19
(binary 19). The kit joins the chapters' video by stream copy with an h264_metadata filter
(crates/explainer_kit/src/master.rs:193-251), so the joins depend on each chapter's parameter sets.
Against the TRJ production19 chain (trj-final-verify19-02.py, trj-final-pictures182.py, both
preserved), 321 runs on CSD and adds:
  - provenance per chapter from 318's own records: kind, binary (09 = build313's recorded hash, read
    here, never written), staged hard links, and each source job's render summary;
  - the schedule contract for all 15 chapters: each staged table equals timeline269's and production
    timeline21's under that chapter's narration folder, and every line plays one of the 105 retained
    Frederick (ElevenLabs) takes, byte-identical to rebind269's and assembly19's;
  - decoded-frame identity: framemd5 of the master equals the 15 staged chapter movies' framemd5,
    concatenated in registry order, so the joined pictures are exactly the rendered ones;
  - the previous layout anchors (production19-era and 268-era lint findings, table below) placed on
    both clocks, because the older 09 layout stamps run 115.3 s early against the current schedule;
  - 09's review312 L2 frames, preview313's measured zoom-probe exit and its plate sites (a flagged or unlocalized
    site over its whole search window);
  - extraction from the actual final master, after a successful verification job only.
It reuses the reviewed shared checks (verify_stage09_final.py, the strict decode script and caption
screen at their pinned hashes) for captions, phrase marks, media facts, decode lines and loudness.

Modes (python3 -I -B csd-final-verify321.py MODE):
  check            read-only: every static gate and the picture plan's counts. Writes nothing.
  launch-verify    root only: gates, then freeze Q/final-verify321-inputs.json and start one guarded job.
  verify           inside that guard only.
  launch-pictures  root only: requires the verification job's success, then one guarded picture job.
  pictures         inside that guard only.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import wave

VERSION = '321'
FPS = 60
TOTAL_FRAMES = 133662
D = Path('/data/projects/jmev-stroke-probe-20261106')            # CSD
Q = D/'qualification01/production-repair17'
GUARD = D/'guard_csd02.py'
PYTHON = '/usr/bin/python3'
THREADS = 16
# The master, as cc0-combined-master318.py names it (JOB, INPUTS, out = job/'final', master-summary.json,
# staged-inputs.json). Its own hash is taken from the inputs its completed job bound, never written here.
MASTER_HELPER = Q/'cc0-combined-master318.py'
MASTER = D/'jobs/csd-combined-master318'
MASTER_INPUTS = Q/'combined-master318-inputs.json'
OUT = MASTER/'final'
VERIFY = D/'jobs/csd-final-verify321'
VERIFY_INPUTS = Q/'final-verify321-inputs.json'
PICTURES = D/'jobs/csd-final-pictures321'
PICTURES_INPUTS = Q/'final-pictures321-inputs.json'
# Reviewed shared checks: the TRJ production19 verifier's pins (trj-final-verify19-02.py:9-12).
SHARED = Q/'verify_stage09_final.py'
STRICT = Q/'verify_video_preserve_logs01.sh'
SCREEN = Q/'caption-interval-audit01.py'
PLAN182 = Q/'final-picture-plan182.json'                         # production19 master sampling plan
PINS = {SHARED: 'f266a2315e4a9af247c648da416881f3a2999e2682e9c611ff82ca3727b70b94',
        STRICT: '3f4c68dbdcb02c1c90ff03cb4f20afc36792b7f20c6cc111a5eef21617a11410',
        SCREEN: '3cc2643615ce5263b01d429ae6f72326c95e777af3e7f29ef9266a05155fa0c4',
        PLAN182: '20a36f91989e0f1e0ca5149a74483ec22fe3728be44b872fc31a0859747b962a'}
EXPECT = {'EXPECT_RES': '3840x2160', 'EXPECT_FPS': '60', 'EXPECT_BT709': '1', 'EXPECT_LUFS': '-16', 'EXPECT_TP_MAX': '-1.0'}
BINARY_268 = '983954c508e42b90d443ad6ca09f2baba7565a4f609d09d2ea821640534d6ec7'
BINARY_19 = 'f4836eeeb4f2465c0d27ca0497f123f0af6bc0db7bc65096098bb20c5abb0f22'
BUILD268 = D/'jobs/csd-review-repair-build268'
BUILD313 = D/'jobs/csd-review-repair-build313'
REPAIR310_SHA = 'bda7d931498b23bd3203cd0c6106de69dc0a8c5e160e738d698376322ff39e53'
FINAL317 = 'csd-reviewed-final-09-boundary317'
PREVIEW313 = D/'jobs/csd-review-repair-preview313'
NARRATION = D/'jobs/csd-review-repair-rebind269/narration'        # the master's and captions' narration
NARRATION313 = D/'jobs/csd-review-repair-rebind313/narration'
ORIGINAL_NARRATION = D/'jobs/csd-production-assembly19/narration'
TIMELINE269 = D/'jobs/csd-review-repair-timeline269/timeline-output'
BASELINE = D/'jobs/csd-production-timeline21/timeline-output'
SCRIPT = 'explainer_corpus/complex_analysis_visual/script.tsv'
WORKSPACE268 = D/'work/review-repair268/workspace'
WORKSPACE313 = D/'work/review-repair313/workspace'
BOUNDARY = '09_boundary'
CHANGED = ('01_the_bump', '04_reading_color', '05_gallery', '06_one_derivative', '07_one_point', '08_loop_sums',
           '10_rigidity', '11_counting', '12_residues', '14_continuation')
RETAINED = ('02_turning_numbers', '03_two_planes', '13_real_integral', '15_closing')
KINDS = {**{k: 'combined' for k in CHANGED}, **{k: 'retained' for k in RETAINED}, BOUNDARY: 'boundary313'}
SUFFIXES = ('.mp4', '.timeline.tsv', '.cues.tsv', '.cuelog.tsv', '.layout.tsv')
# 09, chapter-local frames: review312 L2 (boundary312-disposition305.json) and review-repair-preview313.py:837-848.
EXIT_WINDOW = range(2084, 2103)
ABSENT_ANCHORS = {2326: 'bound |g| <= M arrival', 4180: 'root 69.67 s', 5417: 'root 90.28 s',
                  6240: 'review312 104 s lower crossing', 6900: 'root grid 115 s', 6960: 'root grid 116 s',
                  7020: 'root grid 117 s'}
PERSISTENCE = range(2340, 7981, 60)                              # review312 L2 coarse 39-133 s
ANCHOR_DELTAS = (-12, 0, 12, 60, 180, 360, 600)                  # trj-final-pictures182.py:36
CLOCK_TOLERANCE = 6                                              # frames: one anchor, not two
# Previous layout anchors: every distinct non-empty finding in these local snapshots (Q-relative paths, sha256).
PREVIOUS_LAYOUT_SOURCES = {
    'final-picture-source41/005-01_the_bump.layout.tsv': 'f47a0de94926bf43544e85891272a3329a2bc6fdb5ae325b66c03e0098d3d9d1',
    'final-picture-source41/006-01_the_bump.layout.tsv': 'f47a0de94926bf43544e85891272a3329a2bc6fdb5ae325b66c03e0098d3d9d1',
    'final-picture-source41/018-04_reading_color.layout.tsv': '8a90110e06cb0b65ae7d3832628a8a38b58f6401281c574c73f832b87f42f650',
    'final-picture-source41/022-05_gallery.layout.tsv': 'a57cf8ac93fc251be3bba32424f429b5f03eda82f8b1ecbb1eb14b3226965a2c',
    'final-picture-source41/037-09_boundary.layout.tsv': 'd380d9db33236ecf73b40f8be6f2aee217cacf66b411f2a88056542a645e1b52',
    'final-picture-source41/038-09_boundary.layout.tsv': '01ec0c401fc962124c06d93de7c8d473894a9dedc643321b103cf9ad673f26e7',
    'final-picture-source41/046-11_counting.layout.tsv': 'a68613a089dc9c3bf5aa6113771464c3959fbee110dbdfd2376907f1e9cf8dad',
    'final-picture-source41/050-12_residues.layout.tsv': 'fbbb5d90a0857ae8fac9b7e5f59931ebb8c581a77a84cccaa952593bb1d0444b',
    'final01-terminal142/01_the_bump.layout.tsv': 'f47a0de94926bf43544e85891272a3329a2bc6fdb5ae325b66c03e0098d3d9d1',
    'final08-terminal218/08_loop_sums.layout.tsv': '7be1c4b12ad55f624040c3217f98587c89c3f94f07807c0f8e7f211d309aa951',
    'final08-terminal64/08_loop_sums.layout.tsv': '6222dce30edc43e77a2990963d2ff42c0d43860c8fed4b252c3310872dd85f25',
    'preview274-review305/preview/01_the_bump.layout.tsv': 'f47a0de94926bf43544e85891272a3329a2bc6fdb5ae325b66c03e0098d3d9d1',
    'preview274-review305/preview/09_boundary.layout.tsv': 'f91bb8a1bb6b7ea895414e739250922aa86eb02f9ecab57f851e95ee06b59a7e',
    'preview243-review265/preview/08_loop_sums.layout.tsv': '7be1c4b12ad55f624040c3217f98587c89c3f94f07807c0f8e7f211d309aa951',
    'preview21-09-review/09_boundary.layout.tsv': '3a76aa106c5b5e80ef5b2e6ae785eb6b1d8d4093e5df801c49417200c9803ae7',
    'preview21-08-review/08_loop_sums.layout.tsv': '6222dce30edc43e77a2990963d2ff42c0d43860c8fed4b252c3310872dd85f25',
    'preview70-review163/preview/08_loop_sums.layout.tsv': 'cd6c992dcdeb6d8f983189326d19ab1efba76a77d1ec97b8cc8772f0d77de503',
    'preview166-review180/preview/08_loop_sums.layout.tsv': '7be1c4b12ad55f624040c3217f98587c89c3f94f07807c0f8e7f211d309aa951',
}
PREVIOUS_LAYOUT = (
    ('01_the_bump', '01_1 +0.8s @0.8s\ttight\t"\\\\frac{1}{1 + x^2}" stands 0.22 from the top edge; a text wants 0.25 (title-safe) (still there at 526 later looks)'),
    ('04_reading_color', '04_4 +7.1s @56.4s\tbehind\ta stroke of an unlabeled shape (x -5.80..0.60, y -0.76..-0.75), painted under "zero of order 2", passes through it near (-2.70, -0.76) (still there at 77 later looks)'),
    ('04_reading_color', 'hold @64.5s\tbehind\ta stroke of an unlabeled shape (x -5.80..0.60, y -3.01..-3.00), painted under "precisely:", passes through it near (-5.67, -3.01) (still there at 41 later looks)'),
    ('04_reading_color', 'hold @64.5s\tbehind\ta stroke of an unlabeled shape (x -5.80..0.60, y -3.08..-3.07), painted under "near a zero z = a of order m, f(", passes through it near (-5.20, -3.08) (still there at 41 later looks)'),
    ('04_reading_color', 'hold @64.5s\tbehind\ta stroke of an unlabeled shape (x -5.80..0.60, y -3.40..-3.39), painted under "m times as z circles a", passes through it near (-4.82, -3.39) (still there at 41 later looks)'),
    ('04_reading_color', '04_7 +9.7s @106.4s\tbehind\ta stroke of an unlabeled shape (x -5.80..0.60, y -3.41..-3.40), painted under "names to search:", passes through it near (-4.75, -3.40) (still there at 24 later looks)'),
    ('05_gallery', '05_2 +8.2s @25.8s\tbehind\ta stroke of an unlabeled shape (x -6.20..-0.60, y -0.72..-0.71), painted under "simple zero at 1", passes through it near (-2.57, -0.71) (still there at 33 later looks)'),
    ('05_gallery', '05_2 +9.6s @27.2s\tbehind\ta stroke of an unlabeled shape (x -6.20..-0.60, y 0.74..0.75), painted under "pole of order 2", passes through it near (-5.21, 0.75) (still there at 26 later looks)'),
    ('05_gallery', '05_4 +10.8s @58.6s\tbehind\ta stroke of an unlabeled shape (x -6.60..1.80, y 0.65..0.66), painted under "\\\\sin(i) \\\\approx 1.18\\\\,i", passes through it near (-0.90, 0.66) (still there at 29 later looks)'),
    ('05_gallery', '05_4 +13.8s @61.6s\tbehind\ta stroke of an unlabeled shape (x -6.60..1.80, y 1.66..1.67), painted under "\\\\sin(3i) \\\\approx 10.0\\\\,i", passes through it near (-0.95, 1.66) (still there at 14 later looks)'),
    ('05_gallery', '05_6 +7.0s @86.2s\tbehind\ta stroke of an unlabeled shape (x -6.60..1.80, y 1.64..1.65), painted under "integral works to the right", passes through it near (-1.42, 1.64) (still there at 81 later looks)'),
    ('09_boundary', '09_3 +6.9s @41.6s\tbehind\ta stroke of an unlabeled shape (x 2.80..2.82, y -1.02..1.62), painted under "opposite samples share a value", passes through it near (2.81, -0.76) (still there at 222 later looks)'),
    ('09_boundary', '09_7 +0.6s @108.0s\ttight\t"x" stands 0.18 from the right edge; a text wants 0.25 (title-safe) (still there at 255 later looks)'),
    ('09_boundary', '09_1 +2.6s @2.6s\tbehind\ta stroke of an unlabeled shape (x -6.10..-0.50, y -0.38..-0.37), painted under "a", passes through it near (-3.10, -0.38) (still there at 658 later looks)'),
    ('09_boundary', '09_8 +1.8s @133.4s\tbehind\ta stroke of an unlabeled shape (x -6.10..-0.50, y -0.38..-0.37), painted under "a", passes through it near (-2.22, -0.38) (still there at 126 later looks)'),
    ('11_counting', '11_1 +6.0s @6.0s\tbehind\ta stroke of an unlabeled shape (x -6.25..-0.65, y -2.08..-2.07), painted under "simple zero", passes through it near (-3.93, -2.08) (still there at 23 later looks)'),
    ('11_counting', '11_1 +12.4s @12.4s\tbehind\ta stroke of an unlabeled shape (x -6.25..-0.65, y -2.12..-2.11), painted under "zero of order 2", passes through it near (-2.72, -2.11) (still there at 11 later looks)'),
    ('11_counting', '11_1 +16.4s @16.4s\tbehind\ta stroke of an unlabeled shape (x -6.25..-0.65, y -2.08..-2.07), painted under "simple pole", passes through it near (-3.93, -2.08) (still there at 14 later looks)'),
    ('11_counting', '11_6 +7.0s @107.8s\tbehind\ta stroke of an unlabeled shape (x -6.25..-0.65, y -2.50..-2.49), painted under "one root inside, at distance 0.3", passes through it near (-4.22, -2.50) (still there at 80 later looks)'),
    ('11_counting', '11_7 +7.5s @131.6s\tbehind\ta stroke of an unlabeled shape (x -6.25..-0.65, y -2.70..-2.69), painted under "same five roots", passes through it near (-3.60, -2.69) (still there at 29 later looks)'),
    ('12_residues', '12_4 +3.0s @53.0s\tbehind\ta stroke of an unlabeled shape (x -5.90..-0.90, y 1.04..1.04), painted under "i", passes through it near (-2.65, 1.04) (still there at 56 later looks)'),
    ('12_residues', '12_4 +3.0s @53.0s\tbehind\ta stroke of an unlabeled shape (x -5.90..-0.90, y -1.47..-1.46), painted under "−i", passes through it near (-2.56, -1.46) (still there at 56 later looks)'),
    ('08_loop_sums', '08_5 +143.1s @205.6s\tcrossed\ta stroke of an unlabeled shape (x 2.08..2.81, y 0.73..1.26), painted over "4^n", passes through it near (2.38, 0.95) (still there at 4 later looks)'),
    ('08_loop_sums', '08_5 +143.1s @205.6s\tcrossed\ta stroke of an unlabeled shape (x 4.14..4.86, y 0.23..0.77), painted over "4^n", passes through it near (4.43, 0.45) (still there at 4 later looks)'),
    ('08_loop_sums', '08_5 +54.1s @116.6s\tbehind\ta stroke of an unlabeled shape (x -2.61..-1.59, y -0.12..0.91), painted under "c", passes through it near (-1.88, 0.90) (still there at 226 later looks)'),
    ('09_boundary', '09_3 +6.8s @156.8s\tbehind\ta stroke of an unlabeled shape (x 2.80..2.82, y -1.02..1.62), painted under "opposite samples share a value", passes through it near (2.81, -0.76) (still there at 223 later looks)'),
    ('09_boundary', '09_7 +0.5s @223.2s\ttight\t"x" stands 0.18 from the right edge; a text wants 0.25 (title-safe) (still there at 255 later looks)'),
    ('09_boundary', '09_1 +57.4s @57.4s\tcrossed\ta stroke of an unlabeled shape (x 4.50..5.07, y -1.66..-1.37), painted over "\\\\rho", passes through it near (4.71, -1.48) (still there at 53 later looks)'),
    ('09_boundary', '09_1 +71.8s @71.8s\tbehind\ta stroke of an unlabeled shape (x 3.91..3.99, y -1.48..-1.41), painted under "p", passes through it near (3.95, -1.44) (still there at 47 later looks)'),
    ('08_loop_sums', '08_5 +90.1s @152.6s\tcrossed\ta stroke of an unlabeled shape (x -2.50..-1.83, y 0.25..0.92), painted over "c", passes through it near (-1.87, 0.71) (still there at 47 later looks)'),
)
FLAGS = dict(final_pictures_reviewed=False, motion_reviewed=False, listening_accepted=False, fine_sync_accepted=False,
             native_4k_detail_reviewed=False, independent_av_accepted=False, publication_accepted=False)


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def success(path):
    r = read(path)
    assert r.get('admitted') and r.get('finished') and r.get('exit') == 0, path
    assert r['inputs_sha256'] == r['final_inputs_sha256'], path
    assert not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), path
    return r


def stable(job, manifest):
    """A finished guarded job whose own inputs are the ones its receipt bound, every row unchanged."""
    receipt = success(job/'resource-receipt.json')
    assert sha(manifest) == receipt['inputs_sha256'], 'not the inputs '+job.name+' bound'
    files = read(manifest)['files']
    rows = {x['path']: x['sha256'] for x in files}
    assert len(rows) == len(files)
    for path, digest in rows.items():
        assert sha(path) == digest, 'bound input of '+job.name+' changed: '+path
    return receipt, rows


def shared():
    for path, digest in PINS.items():
        assert sha(path) == digest, 'not the pinned '+path.name
    spec = importlib.util.spec_from_file_location('verify_stage09_final', SHARED)
    v = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(v)
    assert v.EXPECT == EXPECT and v.FPS == FPS
    assert v.PINS[v.STRICT_REL] == PINS[STRICT] and v.PINS[v.SCREEN_REL] == PINS[SCREEN]
    return v


def build313():
    success(BUILD313/'resource-receipt.json')
    q = read(BUILD313/'qualification.json')
    assert q['base_binary_sha256'] == BINARY_268 and q['binary_sha256'] != BINARY_268
    assert q['changed'] == ['explainer_corpus/complex_analysis_visual/src/chapters/boundary.rs']
    return q['binary_sha256']


def master318():
    """318's completed job and its own records: registry, kinds, binaries, replays, outputs, hard links."""
    receipt, rows = stable(MASTER, MASTER_INPUTS)
    # 318 launch: guard ... -- python3 -I -B <318 resolved> master --job J --inputs I --manifest M (cc0-combined-master318.py:812-814)
    helper = MASTER_HELPER.resolve()
    command = [str(x) for x in receipt['command']]
    assert command[3:9] == [str(helper), 'master', '--job', MASTER.name, '--inputs', str(MASTER_INPUTS)], command
    assert command[9] == '--manifest' and len(command) == 11, command
    assert rows.get(str(helper)) == sha(helper)
    success(BUILD268/'resource-receipt.json')
    registry = read(BUILD268/'qualification.json')['chapters']
    assert len(registry) == len(set(registry)) == 15 and set(registry) == set(KINDS)
    summary = read(MASTER/'master-summary.json')
    assert (summary['version'], summary['host'], summary['frames'], summary['chapters']) == ('318', 'csd', TOTAL_FRAMES, registry)
    assert summary['consumer_binary_sha256'] == BINARY_268 and summary['narration'] == str(NARRATION)
    assert summary['native_bit_identical_replays'] == 15 and list(summary['replays']) == registry
    manifest = summary['manifest']
    assert manifest['path'] == command[10] and rows.get(manifest['path']) == manifest['sha256'], 'not the manifest 318 bound'
    binary313 = build313()
    want = {'combined': BINARY_268, 'retained': BINARY_19, 'boundary313': binary313}
    by = {r['chapter']: r for r in summary['rows']}
    assert [r['chapter'] for r in summary['rows']] == registry
    for key in registry:
        assert by[key]['kind'] == KINDS[key] and by[key]['binary_sha256'] == want[KINDS[key]], key
    assert summary['chapter_binaries'] == {k: by[k]['binary_sha256'] for k in registry}
    assert by[BOUNDARY]['job'] == FINAL317 and by[BOUNDARY]['narration'] == str(NARRATION313)
    b = summary['boundary_replay']
    assert (b['chapter'], b['bit_identical'], b['captured_by_binary_sha256'], b['replayed_by_binary_sha256']) == \
        (BOUNDARY, True, binary313, BINARY_268)
    outputs = {}
    for key, name in (('master', '_master.mp4'), ('srt', '.en.srt'), ('vtt', '.en.vtt')):
        item = summary['outputs'][key]
        assert item['path'] == str(OUT/('complex_analysis_visual'+name)) and sha(item['path']) == item['sha256'], key
        outputs[key] = Path(item['path'])
    assert set(summary['outputs']) == set(outputs)
    links = read(MASTER/'staged-inputs.json')
    expected = {OUT/(k+s) for k in registry for s in SUFFIXES} | {OUT/'soundtrack'/k/'soundtrack_0.wav' for k in registry}
    assert len(links) == len(expected) and {Path(x['path']) for x in links} == expected
    sources, digest = {}, {}
    for x in links:
        # a hard link of a source that stable() just re-hashed as a bound row: same inode, same bytes
        source, staged = Path(x['source']), Path(x['path'])
        assert os.path.samefile(source, staged) and x['sha256'] == rows.get(str(source)), x['path']
        sources[staged], digest[staged] = source, x['sha256']
    files = {MASTER_HELPER, MASTER_INPUTS, *map(Path, rows), BUILD268/'resource-receipt.json', BUILD268/'qualification.json',
             BUILD313/'resource-receipt.json', BUILD313/'qualification.json'}
    files.update(p for p in MASTER.rglob('*') if p.is_file())
    capture = OUT/'soundtrack'/BOUNDARY/'soundtrack_0.wav'
    assert (b['capture']['path'], b['capture']['source'], b['capture']['sha256']) == (str(capture), str(sources[capture]), digest[capture])
    for key in registry:
        row, movie = by[key], sources[OUT/(key+'.mp4')]
        # 318 chapter_gate: source = host/'jobs'/job, render/<key>.mp4 (foreign hosts at their original paths after import)
        job = Path(row['host'])/'jobs'/row['job']
        assert movie == job/'render'/(key+'.mp4') and digest[OUT/(key+'.mp4')] == row['movie_sha256'], key
        render = read(job/'render-summary.json')
        assert (render['chapter'], render['frames'], render['binary_sha256']) == (key, row['frames'], row['binary_sha256']), key
        listed = {x['path']: x['sha256'] for x in render['files']}
        for staged in [OUT/(key+s) for s in SUFFIXES] + [OUT/'soundtrack'/key/'soundtrack_0.wav']:
            assert listed.get(str(sources[staged])) == digest[staged], str(sources[staged])
        if key == BOUNDARY:
            assert render['boundary_sha256'] == REPAIR310_SHA and render['build313_qualification_sha256'] == sha(BUILD313/'qualification.json')
        with wave.open(str(OUT/'soundtrack'/key/'soundtrack_0.wav')) as w:
            assert (w.getframerate(), w.getnchannels(), w.getsampwidth(), w.getnframes()) == (48000, 2, 2, row['frames']*800), key
        cues = [x for x in (OUT/(key+'.cuelog.tsv')).read_text().splitlines() if x.strip() and not x.startswith('#')]
        assert summary['replays'][key] == dict(cues=len(cues), samples=row['frames']*800), key
        files.update((job/'render-summary.json', job/'resource-receipt.json'))
    return dict(registry=registry, by=by, outputs=outputs, sources=sources, summary=summary,
                binary313=binary313, files=files)


def schedules(v, m, script):
    """Every staged table is timeline269's and production timeline21's under its chapter's narration folder;
    every line plays a retained take byte-identical to rebind269's and assembly19's; the cues follow the script."""
    offset, chapters, placements, timelines, files = 0, [], [], {}, set()
    for key in m['registry']:
        row = m['by'][key]
        narration = row['narration']
        assert row['start_frame'] == offset, key+': 318 placed it elsewhere'
        for kind in ('timeline', 'cues'):
            name = key+'.'+kind+'.tsv'
            new = (OUT/name).read_bytes()
            assert new == (TIMELINE269/name).read_bytes().replace(str(NARRATION).encode(), narration.encode()), name+' differs from timeline269'
            assert new == (BASELINE/name).read_bytes().replace(str(ORIGINAL_NARRATION).encode(), narration.encode()), name+' differs from timeline21'
            files.update((TIMELINE269/name, BASELINE/name))
        name = key+'.cuelog.tsv'
        base = (TIMELINE269/name).read_text().replace(str(NARRATION), narration).splitlines()
        assert (OUT/name).read_text().splitlines()[:len(base)] == base, name
        files.add(TIMELINE269/name)
        t = v.parse_timeline((OUT/(key+'.timeline.tsv')).read_text(), key)
        assert (t['chapter'], t['fps'], t['frames']) == (key, FPS, row['frames']), key
        for line in t['lines']:
            wav = Path(line['wav'])
            assert str(wav.parent) == narration and wav.name == line['id']+'.wav', line['wav']
            assert sha(wav) == sha(NARRATION/wav.name) == sha(ORIGINAL_NARRATION/wav.name), 'not the retained take: '+line['wav']
            seconds = v.wav_seconds(wav)
            assert line['start']/FPS + seconds <= t['frames']/FPS + 0.001, line['id']+' runs past '+key
            placements.append(dict(cue=line['id'], chapter=key, start_frame=offset+line['start'], duration_s=seconds))
            files.update((wav, NARRATION/wav.name, ORIGINAL_NARRATION/wav.name))
        chapters.append(dict(chapter=key, kind=row['kind'], start_frame=offset, frames=t['frames'],
                             binary_sha256=row['binary_sha256'], timeline_sha256=sha(OUT/(key+'.timeline.tsv'))))
        timelines[key] = t
        offset += t['frames']
    assert offset == TOTAL_FRAMES == m['summary']['frames']
    assert [p['cue'] for p in placements] == list(script) and len(placements) == 105
    wavs = sorted(NARRATION.glob('*.wav'))
    assert len(wavs) == 105 and {w.stem for w in wavs} == {p['cue'] for p in placements}
    for wav in wavs:
        assert sha(wav) == sha(NARRATION313/wav.name) == sha(ORIGINAL_NARRATION/wav.name), wav.name
    return chapters, placements, timelines, offset, files


def film(v):
    """All static gates. Returns what verify and pictures use, plus every file they read."""
    m = master318()
    script_path = WORKSPACE268/SCRIPT
    assert sha(script_path) == sha(WORKSPACE313/SCRIPT)
    script = v.read_script(script_path)
    assert len(script) == 105
    chapters, placements, timelines, total, files = schedules(v, m, script)
    report = read(NARRATION/'report.json')
    captions = v.check_captions(m['outputs']['srt'].read_text(), m['outputs']['vtt'].read_text(), script, report, placements, total)
    marks = v.check_phrases(read(NARRATION/'phrase-timings.json'), script, NARRATION)
    files |= m['files'] | {script_path, WORKSPACE313/SCRIPT, GUARD, Path(__file__).resolve(), *PINS}
    for folder in (NARRATION, NARRATION313, ORIGINAL_NARRATION):
        files.update(p for p in folder.rglob('*') if p.is_file())
    return dict(m=m, script=script, chapters=chapters, placements=placements, timelines=timelines, total=total,
                captions=captions, phrase_marked_cues=len(marks), files=files)


def anchors(row, starts, frames):
    """A layout finding's chapter-local frames: its @t stamp and, when the head names a cue, that cue's start plus
    its offset. Both are kept when they disagree (the older 09 stamps sit 115.3 s early; first-cue rows count
    their offset from the chapter start)."""
    head = row.split('\t', 1)[0]
    m = re.fullmatch(r'(?:(\d\d_\d+) \+([0-9]+(?:\.[0-9]+)?)s|[^@]*?) ?@([0-9]+(?:\.[0-9]+)?)s', head)
    assert m, 'unrecognized layout finding: '+row[:90]
    at = round(float(m.group(3))*FPS)
    found = [(at, '@t')]
    if m.group(1) and m.group(1) in starts:
        cue = starts[m.group(1)] + round(float(m.group(2))*FPS)
        if abs(cue - at) > CLOCK_TOLERANCE:
            found.append((cue, 'cue start + offset'))
    return [(f, clock) for f, clock in found if 0 <= f < frames]


def picture_plan(f, layouts, plan182, preview):
    """Plan182's reviewed samples, current and previous layout anchors on both clocks, 09's review312 L2 and
    preview313 frames, chapter seams, and 10 s periodic frames; native 4K frames as a separate small list."""
    by = {c['chapter']: c for c in f['chapters']}
    assert [(c['chapter'], c['start_frame'], c['frames']) for c in plan182['chapters']] == \
        [(c['chapter'], c['start_frame'], c['frames']) for c in f['chapters']], 'plan182 chapters differ from the master'
    samples, natives = {}, {}

    def add(table, key, local, reason):
        c = by[key]
        if 0 <= local < c['frames']:
            x = table.setdefault(c['start_frame']+local, dict(chapter=key, local_frame=local, reasons=[]))
            if reason not in x['reasons']:
                x['reasons'].append(reason)

    for x in plan182['samples']:
        assert x['frame'] == by[x['chapter']]['start_frame'] + x['local_frame'], x
        for reason in x['reasons']:
            add(samples, x['chapter'], x['local_frame'], 'plan182: '+reason)
    for x in plan182['native_detail_samples']:
        assert x['frame'] == by[x['chapter']]['start_frame'] + x['local_frame'], x
        for reason in x['reasons']:
            add(natives, x['chapter'], x['local_frame'], 'plan182: '+reason)
    rows = [(k, r, 'current layout') for k in by for r in layouts[k] if r.strip()]
    rows += [(k, r, 'previous layout') for k, r in PREVIOUS_LAYOUT]
    for key, row, label in rows:
        c = by[key]
        starts = {x['id']: x['start'] for x in f['timelines'][key]['lines']}
        for anchor, clock in anchors(row, starts, c['frames']):
            for delta in ANCHOR_DELTAS:
                add(samples, key, anchor+delta, '%s (%s %+d): %s' % (label, clock, delta, row))
            if 'later looks' in row:
                for local in range(anchor, c['frames'], 300):
                    add(samples, key, local, '%s persistence (%s): %s' % (label, clock, row.split('\t', 1)[0]))
            if label == 'current layout':
                add(natives, key, anchor, 'current layout native 4K (%s): %s' % (clock, row.split('\t', 1)[0]))
    for local in EXIT_WINDOW:
        add(samples, BOUNDARY, local, 'zoom-line exit: probe window 2086-2100 enforced (312 by eye 2087-2099), +-2')
    for local, why in ABSENT_ANCHORS.items():
        for delta in (0, 30, 60):
            add(samples, BOUNDARY, local+delta, 'review312/preview313 anchor %d %+d: %s' % (local, delta, why))
        add(natives, BOUNDARY, local, 'native 4K at preview313 anchor %d: %s' % (local, why))
    for local in PERSISTENCE:
        add(samples, BOUNDARY, local, 'review312 L2: coarse persistence 39-133 s')
    exit_frame = preview['summary']['chapters'][BOUNDARY]['zoom_probe_exit_frame']
    for local in range(exit_frame-2, exit_frame+16):
        add(samples, BOUNDARY, local, 'preview313 zoom probe exit %d' % exit_frame)
    add(natives, BOUNDARY, exit_frame, 'native 4K at preview313 zoom probe exit %d' % exit_frame)
    for x in preview['sampling']['samples']:
        add(samples, BOUNDARY, x['frame'], 'preview313 pick: '+x['reasons'][0])
    # preview313's plate sites (sampling.json fades, review-repair-preview313.py:656-718, :1058-1059). No gate reads
    # their flags (review320 observation 2), so the master shows each placed site and every flagged or unlocalized
    # site's whole search window.
    for fade in preview['sampling']['fades']:
        label = 'preview313 plate site %s (%s%s)' % (fade['id'], fade['how'], ', flagged' if fade.get('flag') else '')
        if fade['start'] is not None:
            span = fade['frames']*fade['repeat']
            for local in (fade['start'], fade['start']+span//2, fade['start']+span-1, fade['start']+span+30):
                add(samples, BOUNDARY, local, label)
        if 'search' in fade and (fade['start'] is None or fade.get('flag')):
            lo, hi = fade['search']
            for local in range(lo, hi+1, 30):
                add(samples, BOUNDARY, local, label+': search window')
    for key, c in by.items():
        for d in (0, 1, 2):
            add(samples, key, d, 'chapter first frames (seam)')
            add(samples, key, c['frames']-1-d, 'chapter last frames (seam)')
        for local in range(0, c['frames'], 600):
            add(samples, key, local, 'periodic 10 s')

    def listed(table):
        return [dict(frame=g, seconds=g/FPS, **x) for g, x in sorted(table.items())]
    return listed(samples), listed(natives)


def preview313():
    """Only its frame picks and probe exit are used: a successful receipt over the manifest it bound (its rows
    were re-verified by 317 and 318 before the 09 final and the master)."""
    receipt = success(PREVIEW313/'resource-receipt.json')
    assert sha(Q/'review-repair-preview313-inputs.json') == receipt['inputs_sha256']
    summary, sampling = read(PREVIEW313/'summary.json'), read(PREVIEW313/BOUNDARY/'sampling.json')
    assert summary['chapters'][BOUNDARY]['zoom_probe_fails'] == []
    files = {PREVIEW313/'resource-receipt.json', Q/'review-repair-preview313-inputs.json', PREVIEW313/'summary.json',
             PREVIEW313/BOUNDARY/'sampling.json'}
    return dict(summary=summary, sampling=sampling, files=files)


def frame_hashes(path):
    """ffmpeg framemd5 rows `stream, dts, pts, duration, size, hash`, video stream 0 only."""
    out = []
    for line in Path(path).read_text().splitlines():
        if not line.strip() or line.startswith('#'):
            continue
        cells = [x.strip() for x in line.split(',')]
        assert len(cells) == 6 and cells[0] == '0', line
        out.append(cells[5])
    return out


def admitted(job, inputs, mode):
    r = read(job/'resource-receipt.json')
    assert r['admitted'] and 'exit' not in r
    assert r['policy']['stop_free_gib'] == 8 and r['policy']['phase_growth_gib'] == 4
    command = [str(x) for x in r['command']]
    assert str(Path(__file__).resolve()) in command and mode in command, command
    assert sha(inputs) == r['inputs_sha256']
    frozen = read(inputs)['files']
    bound = {x['path']: x['sha256'] for x in frozen}
    assert len(bound) == len(frozen)
    for path, digest in bound.items():
        assert sha(path) == digest, path
    available = sorted(os.sched_getaffinity(0))
    assert len(available) >= THREADS
    os.sched_setaffinity(0, set(available[:THREADS]))
    os.environ.update(PATH='/usr/bin:/bin', XDG_CACHE_HOME=str(D/'work/cache'))
    return bound


def require(bound, paths):
    for p in paths:
        assert bound.get(str(p)) == sha(p), 'not bound at its hash: '+str(p)


def run(argv, log, timeout):
    with Path(log).open('x') as stream:
        subprocess.run([str(x) for x in argv], stdout=stream, stderr=subprocess.STDOUT, check=True, timeout=timeout)


def verify(args):
    v = shared()
    bound = admitted(VERIFY, VERIFY_INPUTS, 'verify')
    f = film(v)
    require(bound, f['files'])
    job, master = VERIFY, f['m']['outputs']['master']
    for name in ('strict-video.log', 'decode-logs', 'framemd5', 'encoded-loudness.log', 'caption-screen.json', 'verification.json'):
        assert not (job/name).exists(), name
    # full decode: root's strict check (software with -xerror, then the hardware second opinion), EXPECT_* gates
    (job/'decode-logs').mkdir()
    env = dict(os.environ, **EXPECT, TMPDIR=str(job/'decode-logs'))
    with (job/'strict-video.log').open('x') as log:
        subprocess.run(['bash', str(STRICT), str(master)], stdout=log, stderr=subprocess.STDOUT, env=env, check=True, timeout=14400)
    strict = (job/'strict-video.log').read_text(errors='replace')
    assert strict.splitlines()[-1] == 'VERIFY PASS'
    decodes = v.decode_lines(strict)
    assert v.software_decode_clean(decodes)
    assert any((job/'decode-logs').glob('verify_decode.*'))
    # frame and audio contract: streams, packet count, picture and audio length
    probe = json.loads(subprocess.check_output(['ffprobe', '-v', 'error', '-show_streams', '-show_format', '-of', 'json', str(master)], timeout=600))
    packets = int(subprocess.check_output(['ffprobe', '-v', 'error', '-select_streams', 'V:0', '-count_packets', '-show_entries',
                                           'stream=nb_read_packets', '-of', 'csv=p=0', str(master)], timeout=3600).strip())
    media = v.media_facts(probe, packets, f['total'])
    # decoded-frame identity: the master's pictures are the 15 staged chapter renders, in registry order
    (job/'framemd5').mkdir()
    decode = ['ffmpeg', '-nostdin', '-hide_banner', '-v', 'error', '-xerror', '-threads', str(THREADS)]
    hashes, extradata = {}, {}
    for key in f['m']['registry']:
        movie = OUT/(key+'.mp4')
        run(decode+['-i', movie, '-map', '0:v:0', '-an', '-f', 'framemd5', '-n', job/'framemd5'/(key+'.framemd5')],
            job/'framemd5'/(key+'.log'), 3600)
        hashes[key] = frame_hashes(job/'framemd5'/(key+'.framemd5'))
        c = next(x for x in f['chapters'] if x['chapter'] == key)
        assert len(hashes[key]) == c['frames'], key
    run(decode+['-i', master, '-map', '0:v:0', '-an', '-f', 'framemd5', '-n', job/'framemd5'/'master.framemd5'], job/'framemd5'/'master.log', 14400)
    whole = frame_hashes(job/'framemd5'/'master.framemd5')
    joined = [h for key in f['m']['registry'] for h in hashes[key]]
    if whole != joined:
        k = next((i for i, (a, b) in enumerate(zip(whole, joined)) if a != b), min(len(whole), len(joined)))
        c = next((x for x in f['chapters'] if x['start_frame'] <= k < x['start_frame']+x['frames']), None)
        raise AssertionError('master frame %d (%s local %s) differs from the staged chapter render; %d vs %d frames' % (
            k, c and c['chapter'], c and k-c['start_frame'], len(whole), len(joined)))
    for key, movie in [(k, OUT/(k+'.mp4')) for k in f['m']['registry']] + [('master', master)]:
        streams = json.loads(subprocess.check_output(['ffprobe', '-v', 'error', '-select_streams', 'v:0', '-show_streams',
                                                      '-show_data_hash', 'MD5', '-of', 'json', str(movie)], timeout=600))['streams']
        extradata[key] = streams[0].get('extradata_hash')
    with (job/'encoded-loudness.log').open('x') as log:
        subprocess.run(['ffmpeg', '-nostdin', '-hide_banner', '-nostats', '-i', str(master), '-vn', '-af', 'ebur128=peak=true', '-f', 'null', '-'],
                       stdout=log, stderr=subprocess.STDOUT, check=True, timeout=14400)
    loudness = v.loudness_summary((job/'encoded-loudness.log').read_text(errors='replace'))
    assert float(loudness['true_peak_dbtp']) <= -1
    srt = f['m']['outputs']['srt']
    subprocess.run([PYTHON, '-I', '-B', str(SCREEN), str(srt), str(f['total']), str(FPS), str(job/'caption-screen.json')],
                   check=True, capture_output=True, text=True, timeout=600)
    screen = read(job/'caption-screen.json')
    assert screen['srt_sha256'] == sha(srt)
    for path, digest in bound.items():
        assert sha(path) == digest, path
    result = dict(version=VERSION, master=str(master), master_sha256=sha(master), srt_sha256=sha(srt),
                  vtt_sha256=sha(f['m']['outputs']['vtt']), master_job=MASTER.name,
                  master_helper_sha256=sha(MASTER_HELPER), master_inputs_sha256=sha(MASTER_INPUTS),
                  frames=f['total'], fps=FPS, chapters=f['chapters'], placements=f['placements'],
                  retained_takes=dict(cues=len(f['placements']), byte_identical_to=[str(NARRATION), str(ORIGINAL_NARRATION)]),
                  captions=f['captions'], phrase_marked_cues=f['phrase_marked_cues'],
                  strict_video_check='passed', strict_video_decodes=decodes,
                  strict_video_notes=[x for x in strict.splitlines() if x.startswith('note:')], media=media,
                  decoded_frames_identical_to_staged_chapters=True, decoded_frames=len(whole),
                  extradata_md5=extradata, encoded_loudness=loudness,
                  caption_screen=dict(caption_count=screen['caption_count'],
                                      over_20_characters_per_second=len(screen['over_20_characters_per_second']),
                                      lines_over_42_characters=len(screen['lines_over_42_characters'])),
                  **FLAGS, scope='Technical verification of the 318 master only: decode, frame identity, schedule, '
                  'retained takes, exact captions and loudness. Pictures, motion, listening, fine sync and '
                  'independent AV acceptance remain root\'s.')
    with (job/'verification.json').open('x') as stream:
        json.dump(result, stream, indent=2)
    print('FINAL VERIFY PASS: 318 master decodes cleanly, equals its 15 staged chapters frame for frame, and its captions '
          'equal the script; every picture still needs root\'s viewing.', flush=True)
    return 0


def verified(f):
    """The verification job finished, its inputs held, and its record names this master and its checks."""
    _, rows = stable(VERIFY, VERIFY_INPUTS)
    record = read(VERIFY/'verification.json')
    assert record['master_sha256'] == sha(f['m']['outputs']['master']) and record['frames'] == f['total']
    assert record['strict_video_check'] == 'passed' and record['decoded_frames_identical_to_staged_chapters'] is True
    assert (VERIFY/'strict-video.log').read_text().splitlines()[-1] == 'VERIFY PASS'
    assert all(record[k] is False for k in FLAGS)
    return {VERIFY_INPUTS, *map(Path, rows), *(p for p in VERIFY.rglob('*') if p.is_file())}


def plan_for(f):
    layouts = {k: (OUT/(k+'.layout.tsv')).read_text().splitlines() for k in f['m']['registry']}
    preview = preview313()
    samples, natives = picture_plan(f, layouts, read(PLAN182), preview)
    assert samples and natives and all(0 <= x['frame'] < f['total'] for x in samples + natives)
    return samples, natives, preview['files']


def balanced(xs):
    if len(xs) == 1:
        return xs[0]
    k = len(xs)//2
    return '('+balanced(xs[:k])+'+'+balanced(xs[k:])+')'


def pictures(args):
    from PIL import Image, ImageDraw
    v = shared()
    bound = admitted(PICTURES, PICTURES_INPUTS, 'pictures')
    f = film(v)
    require(bound, f['files'] | verified(f))
    samples, natives, used = plan_for(f)
    require(bound, used)
    job, master = PICTURES, f['m']['outputs']['master']
    with (job/'plan.json').open('x') as stream:
        json.dump(dict(samples=samples, natives=natives), stream, indent=2)
    select = lambda xs: "select='"+balanced(['eq(n,%d)' % x['frame'] for x in xs])+"'"
    script = job/'select.filter'
    with script.open('x') as stream:
        stream.write('[0:v]split=2[a][b];[a]'+select(samples)+',scale=960:540[overview];[b]'+select(natives)+'[detail]')
    (job/'pictures').mkdir()
    (job/'native4k').mkdir()
    subprocess.run(['ffmpeg', '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2', '-i', str(master), '-an',
                    '-filter_complex_script', str(script), '-map', '[overview]', '-vsync', '0', '-q:v', '2',
                    str(job/'pictures'/'frame-%06d.jpg'), '-map', '[detail]', '-vsync', '0', '-c:v', 'png',
                    '-compression_level', '3', str(job/'native4k'/'frame-%06d.png')], check=True, timeout=14400)
    for xs, folder, ext, size in ((samples, 'pictures', 'jpg', (960, 540)), (natives, 'native4k', 'png', (3840, 2160))):
        found = sorted((job/folder).glob('frame-*.'+ext))
        assert len(found) == len(xs), (folder, len(found), len(xs))
        for x, path in zip(xs, found):
            with Image.open(path) as im:
                assert im.size == size, path
            x.update(path=str(path), sha256=sha(path))
    sheets = []
    for k in range(0, len(samples), 8):
        canvas = Image.new('RGB', (1920, 4*565), '#10151d')
        draw = ImageDraw.Draw(canvas)
        for i, x in enumerate(samples[k:k+8]):
            with Image.open(x['path']) as im:
                canvas.paste(im.convert('RGB'), ((i % 2)*960, (i//2)*565))
            more = ' (+%d)' % (len(x['reasons'])-1) if len(x['reasons']) > 1 else ''
            draw.text(((i % 2)*960+5, (i//2)*565+542), '%s %.3fs f%d | %s%s' % (
                x['chapter'], x['local_frame']/FPS, x['frame'], x['reasons'][0][:80], more), fill='white')
        path = job/('sheet-%04d.jpg' % (k//8+1))
        with path.open('xb') as stream:
            canvas.save(stream, format='JPEG', quality=94)
        sheets.append(dict(path=str(path), sha256=sha(path)))
    for path, digest in bound.items():
        assert sha(path) == digest, path
    with (job/'sampling.json').open('x') as stream:
        json.dump(dict(version=VERSION, master_sha256=sha(master), frames=f['total'], chapters=f['chapters'],
                       samples=samples, natives=natives, sheets=sheets, previous_layout_sources=PREVIOUS_LAYOUT_SOURCES,
                       **FLAGS, scope='Pictures of the actual 318 master at 960x540 (sheets) and native 4K PNGs; '
                       'root must view every sheet and native frame. No acceptance is recorded here.'), stream, indent=2)
    print('%d pictures in %d sheets and %d native 4K frames await root\'s viewing.' % (len(samples), len(sheets), len(natives)), flush=True)
    return 0


def launch(job, inputs, mode, files):
    assert not job.exists(), 'never duplicate '+job.name
    assert not inputs.exists(), 'preserve any earlier attempt: '+inputs.name
    assert len(os.sched_getaffinity(0)) >= THREADS, 'fewer than 16 CPUs in the inherited affinity'
    with inputs.open('x') as stream:
        json.dump({'files': [dict(path=str(p), sha256=sha(p)) for p in sorted(files)]}, stream, indent=2)
    print(json.dumps(dict(job=job.name, inputs=str(inputs), inputs_sha256=sha(inputs), files=len(files),
                          master_helper_sha256=sha(MASTER_HELPER), master_inputs_sha256=sha(MASTER_INPUTS))), flush=True)
    helper = Path(__file__).resolve()
    subprocess.run([PYTHON, '-I', '-B', str(GUARD), '--phase', 'native', '--job', job.name, '--inputs', str(inputs), '--',
                    '/usr/bin/timeout', '86400', PYTHON, '-I', '-B', str(helper), mode], check=True)
    success(job/'resource-receipt.json')
    return 0


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=['check', 'launch-verify', 'verify', 'launch-pictures', 'pictures'])
    mode = parser.parse_args().mode
    if mode == 'verify':
        return verify(None)
    if mode == 'pictures':
        return pictures(None)
    v = shared()
    f = film(v)
    if mode == 'check':
        samples, natives, _ = plan_for(f)
        sheets = -(-len(samples)//8)
        # the picture job runs under the guard's native phase (4 GiB growth cap). Upper estimate: the largest local
        # 960x540 JPEG (78 KB) and sheet (897 KB) seen in this lane, and raw RGB size for each native PNG.
        upper = len(samples)*78*1024 + sheets*897*1024 + len(natives)*3840*2160*3
        print(json.dumps(dict(check='ok', frames=f['total'], chapters=[(c['chapter'], c['kind'], c['frames']) for c in f['chapters']],
                              cues=len(f['placements']), captions=f['captions'], picture_samples=len(samples),
                              native_4k=len(natives), sheets=sheets, pictures_growth_upper_estimate_gib=round(upper/2**30, 2))))
        return 0
    if mode == 'launch-verify':
        return launch(VERIFY, VERIFY_INPUTS, 'verify', f['files'])
    files = f['files'] | verified(f)
    files |= plan_for(f)[2]
    return launch(PICTURES, PICTURES_INPUTS, 'pictures', files)


if __name__ == '__main__':
    raise SystemExit(main())
