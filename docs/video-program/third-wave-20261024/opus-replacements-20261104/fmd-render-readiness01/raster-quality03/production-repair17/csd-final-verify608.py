"""608: master606 with native553/521 imported chapters. All actual chapter gates are re-run;
451 decoding, frame identity, caption, loudness and picture-selection functions unchanged.
Prepared only; runtime and independent review remain open. Prior versions preserved.
"""
"""451 successor: exact master447 and SSD442 gallery bindings; original377 checks retained.
Prepared only, not staged or run. Historical lineage below remains unchanged.
"""
"""CSD final technical verification and final picture extraction for the mixed complex master 350,
version 377. Prepared only; nothing here has run. It accepts no picture, motion, listening, fine-sync,
caption-reading or independent AV review; every flag it writes for those is False.

377 is 363 (csd-final-verify363.py, preserved unchanged, reviewed by CC0 as 367) with the coverage repairs root
accepted in its disposition of review 367 (root-verifier367-disposition377.json, pinned with review 367's hash):
  - 367 M1: each sampled_preview row's review_scope.playback_seconds are carried out of 350's row. Each instant s
    is sampled at round(s*60) and 2 frames either side, and round(s*60) is a native 4K frame: 07's rule, now also
    for permit 351's four 09 instants (root 343);
  - 367 L1: root 359 (derivative-sheets359-review.json) is pinned. Its 06 viewed_intervals 9630-9730 and
    9970-10029, and the 08 run 10200-10380 that disposition 377 retains, must be among the sampled intervals of
    those finals, and each is planned outright: every frame sampled, a native inside;
  - 367 L2: root's 11/12/14 frame selection is disposition 377's required_native_frames, pinned. That is 363's
    thirteen frames plus natives 11 2010, 2280, 4050 and 14 1140, for 357 O12, O17, O13 and O9. Its
    additional_overview_endpoints (11 1860, 2160, 3840, 4260; 14 1080, 1200) are overview samples.
Every other gate is 363's, unchanged. In the 363 and 339 text below, read 377 for 363's version, job and input names.

363 is 339 (csd-final-verify339.py, preserved unchanged, reviewed by CC0 as 339) fitted to the actual master:
  - the master is cc0-combined-master350.py's job csd-combined-master350 (318's outputs, names and summary,
    plus each row's review basis). Its 09 row is final344 (csd-reviewed-final-09-boundary344, worker
    csd-reviewed-boundary-final344.py at its frozen sha256) in place of final317. Its ten other changed rows are
    the 352 workers' (csd-reviewed-final352.py or css-final-repair352.py at their frozen sha256, jobs
    *-reviewed-final-<chapter>352). 02/03/13/15 keep their historical production19 permits. Every row's review
    basis must be the one 350 records: sampled_preview for 09 and the ten, historical_permit for the four;
  - M1 carried to the new review basis: every frame of every sampled interval a sampled_preview row's final rests
    on (its render summary's review_scope, equal to 350's row) is a final overview sample, with a native 4K frame
    in each interval (06: root's 359 consecutive frames; 08: the preview243 run 10200-10380);
  - frames root requires in the native final, each an overview sample and a native 4K frame: 06 600, 900, 1200,
    1290, 1320, 1680, 1890 (root's disposition 362 of review 270 R1/R2); and, named by root after CC1's review 357,
    11 4348, 4408, 7448, 8121, 8303, 9731, 11880, 12 2532, 2766, 3002 and 14 10260, 10278, 10301;
  - the 347 delivery captions are validated separately from the native caption proof, which is unchanged: the
    master's own SRT/VTT must be byte-identical to native327 (the captions 347 was cut from and root 354 adopted
    it against), and the 347 SRT/VTT must pass the same pinned exact-caption check and caption screen on this
    master's schedule. Root's caption playback check stays open.
Every decode, frame-identity, schedule, take, loudness, caption and picture gate below is 339's, unchanged. The
overview anchor is handled by root's external ledger, as review 339 L1/N1 sets out (notes 363 §5).

339 is 330 (csd-final-verify330.py, preserved unchanged) with one fix, for CC0's independent review 332: 330 bound
the overview job's pictures and sheets at whatever bytes they had at the native launch, so a picture or sheet changed
after the overview job finished was accepted. part_done() now compares every recorded picture and sheet with the
sha256 the overview job wrote into its sampling.json, and requires the folders to hold exactly those files, before
anything is bound; the native job repeats it. A picture forged together with its recorded hash (332 C7) cannot be seen
from inside the folder, so the overview job prints its sampling.json sha256 last, the native launch prints the one it
binds, and the native union records it: root compares the two. Everything else is 330's, with 339's names.

330 was 321 (csd-final-verify321.py, preserved unchanged) with review 323's M1 and L1-L4 fixed, and nothing else:
  M1  root's 305 and CC1's 319 frame-window reviews and root's five 07 playback instants are carried into the final
      samples (every window frame; each instant +-2), with one native 4K frame per window (05: each 319 site's
      full-plate frame). The records are pinned. So that no frame is dropped and the guard is unchanged, extraction
      is two guarded jobs (overview, then native), each with an upper growth estimate within 3.5 GiB; the native job
      proves the two jobs' union is exactly the plan;
  L1  framemd5 of the master and of every chapter must have timebase 1/60, pts 0..n-1 in order and every
      duration 1, so join timing is exact rather than resting on media_facts' rounding margin;
  L2  free space is checked before an inputs file is written (318's 307 L2 precedent);
  L3  reasons are written without local home prefixes, and sheet labels read the anchor, not a path;
  L4  the resolved ffmpeg, ffprobe, bash and python3, and Pillow's files and the libraries its extensions load, are
      bound in both inputs files (launch-final-verify05.py precedent); Pillow is exercised before either inputs file
      is written (pictures-after-preview16.py precedent); and ffmpeg's version line is recorded.
The 318 contract read here is unchanged from 321.

The master (cc0-combined-master318.py, CC0) mixes three binaries: 09_boundary rendered by binary 313
(final317), the ten other changed chapters by binary 268, and 02/03/13/15 retained from production19
(binary 19). The kit joins the chapters' video by stream copy with an h264_metadata filter
(crates/explainer_kit/src/master.rs:193-251), so the joins depend on each chapter's parameter sets.
Against the TRJ production19 chain (trj-final-verify19-02.py, trj-final-pictures182.py, both
preserved), 321 and 330 run on CSD and add:
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

The description of 339 below names the 318 master and final317; for 363 read master350 and final344.

Modes (python3 -I -B csd-final-verify377.py MODE):
  check            read-only: every static gate, the picture plan's counts, sha256 and both jobs' budgets. Writes nothing.
  launch-verify    root only: gates, then freeze Q/final-verify377-inputs.json and start one guarded job.
  verify           inside that guard only.
  launch-pictures  root only: requires the verification job's success, then one guarded picture job for the next
                   part: run it once for overview, then, after that job succeeds, once for native.
  pictures --part overview|native   inside that guard only.
"""
import argparse
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import wave

VERSION = '608'
FPS = 60
TOTAL_FRAMES = 133662
D = Path('/data/projects/jmev-stroke-probe-20261106')            # CSD
Q = D/'qualification01/production-repair17'
GUARD = D/'guard_csd02.py'
PYTHON = '/usr/bin/python3'
THREADS = 16
# The master, as cc0-combined-master350.py names it (JOB, INPUTS, out = job/'final', master-summary.json,
# staged-inputs.json; 318's names with 350's). 363: its frozen hash (CC0, cc0-master350-notes.md) is pinned, and it
# must also be the hash its completed job bound.
MASTER_HELPER = Q/'combined-master606.py'
MASTER_HELPER_SHA = '7c8269fbd291352f4ef9a7828e3539b045f0414dada4870ddc00a057a7b5b163'
MASTER = D/'jobs/csd-combined-master606'
MASTER_INPUTS = Q/'combined-master606-inputs.json'
OUT = MASTER/'final'
VERIFY = D/'jobs/csd-final-verify608'
VERIFY_INPUTS = Q/'final-verify608-inputs.json'
PICTURES = D/'jobs/csd-final-pictures608-%s'                    # M1: one guarded picture job per part, in this order
PICTURES_INPUTS = Q/'final-pictures608-%s-inputs.json'
PICTURE_PARTS = ('overview', 'native')                            # every sample and sheet; every native 4K frame
# L2: the guard admits a native job at 10 GiB free and stops it at 8 GiB; its native phase cap is 4 GiB
# (guard_csd02.py asserts ADMISSION_GIB native 10, STOP_FREE_GIB 8, CAPS native 4). 318 refuses below 12 GiB
# before writing its inputs (cc0-combined-master318.py:183, :807-808); 330 does the same before either inputs file.
GIB = 1024**3
ADMISSION_BYTES = 10*GIB
VERIFY_FREE_BYTES = 12*GIB
# M1: each picture job's upper growth estimate must fit 3.5 GiB, under the guard's unchanged 4 GiB native cap (the
# 0.5 GiB left covers the guard's other counted growth: logs, receipt, the D/work cache). With M1 the whole plan's
# estimate is 3.5-4.3 GiB, so the overview frames and sheets and the native frames are extracted by two jobs.
PICTURE_PART_BYTES = 3*GIB + GIB//2
# L4: the runtime the jobs use, resolved on the guard's PATH and bound in both inputs files
TOOL_PATH = '/usr/bin:/bin'
TOOLS = ('ffmpeg', 'ffprobe', 'bash')
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
FINAL344 = 'csd-reviewed-final-09-boundary344'
CSS = Path('/data/projects/jmev-css-render-worker-20261107')
# 363: the frozen chapter workers master350's rows must name, with the job each runs (%s: the chapter, dashed)
WORKERS = {'boundary313': (Q/'csd-reviewed-boundary-final344.py',
                           'b8458dbcecf3f7a207266dd20195eb3961d82d15a3fc1cc8aa0452db5a2e290b', FINAL344),
           ('combined', D): (Q/'csd-reviewed-final352.py',
                             '23d55c286aba2fc58b77a5af8dbd539293592daa3a4e0e8584f18f324d868760', 'csd-reviewed-final-%s352'),
           ('combined', CSS): (CSS/'qualification01/css-final-repair352.py',
                               '1074051b6063f1cd400de58422f7491d2d011f764884b63c4968b03feaf01a8a', 'css-reviewed-final-%s352')}
REVIEW_BASES = {'combined': 'sampled_preview', 'boundary313': 'sampled_preview', 'retained': 'historical_permit'}
# 363: chapter-local frames root requires in the native final: 06 from root's disposition 362 of review 270 R1/R2
# (its required_final_checks); 11/12/14 named by root after CC1's review 357 (observations O1-O3, O5, O7, O8, O11).
# Each is an overview sample and a native 4K frame; a frame already planned keeps its picture and gains the reason.
# 377: root's disposition of review 367 pins the 11/12/14 selection (367 L2) as required_native_frames, and adds
# overview endpoints; root 359 holds the 06 windows (367 L1). Both are pinned here and checked in plan_for().
DISPOSITION377 = 'root-verifier367-disposition377.json'
REVIEW367 = dict(path='cc0-verifier363-review367.md', sha256='f577a1b36768ac3b59caaec6f96b0790398a683a2525c6d70a72427bc2b62314')
FINAL_FRAME_SOURCES = {'root-preview06-disposition362.json': '1822a5908199c1cf8ae935e50c803e6a419c0109487c4051bef31d0ef0bf0b86',
                       'cc1-chapter-picture-review357.json': '47d51709ed54205c053f9789d5aea194e282a27a8d63305b11d995aa00e71490',
                       DISPOSITION377: 'ecc64557495d90d7762291df7e22284a22f8b98969eb58ad5175a95bc48537ff',
                       'derivative-sheets359-review.json': '40e2fca805e0dfb7b138b2ef68c2770b947bd7ae5e435e3d712bf2ea476d34de'}
FINAL_FRAME_CHECKS = (
    ('06_one_derivative', 'root-preview06-disposition362.json', (600, 900, 1200, 1290, 1320, 1680, 1890),
     'root 362 (270 R1/R2): label edge and label-axis separation'),
    ('11_counting', DISPOSITION377, (4348, 4408, 7448, 8121, 8303, 9731, 11880),
     'root after 357 O1-O3/O5: readout write-in, root marker at chip, held sentence, code type at 4K'),
    ('11_counting', DISPOSITION377, (2010,), 'root after 357 O12: footer top edge meets the plate edge, 1860-2160 centre'),
    ('11_counting', DISPOSITION377, (2280,), 'root after 357 O17: three condition chips stacked, the third on the axis'),
    ('11_counting', DISPOSITION377, (4050,), 'root after 357 O13: inset chip over the output panel border corner, 3840-4260 centre'),
    ('12_residues', DISPOSITION377, (2532, 2766, 3002),
     'root after 357 O7/O8: bottom residue label against player captions'),
    ('14_continuation', DISPOSITION377, (10260, 10278, 10301),
     'root after 357 O11: faint Gamma-outline tint'),
    ('14_continuation', DISPOSITION377, (1140,), 'root after 357 O9: footer chip top edge meets the plate edge, 1080-1200 centre'))
# 377: overview samples only (disposition377 additional_overview_endpoints): both ends of each observation's range
FINAL_ENDPOINT_CHECKS = (
    ('11_counting', DISPOSITION377, (1860, 2160), 'root after 357 O12: observation range ends'),
    ('11_counting', DISPOSITION377, (3840, 4260), 'root after 357 O13: observation range ends'),
    ('14_continuation', DISPOSITION377, (1080, 1200), 'root after 357 O9: observation range ends'))
# 377 (367 L1): windows each final's sampled intervals must contain, also planned outright: every frame, a native inside
REQUIRED_WINDOWS = (
    ('06_one_derivative', 'derivative-sheets359-review.json', (9630, 9730)),
    ('06_one_derivative', 'derivative-sheets359-review.json', (9970, 10029)),
    ('08_loop_sums', DISPOSITION377, (10200, 10380)))
PREVIEW243_MOVIE06 = '5e45d72da91701c9c7c346b98ded8491c13ba1befed6be1b6301c1c6d33ed62d'
# 363: the delivery captions. Native327 is binary 268's `captions all` on the timeline269 tables and rebind269
# narration (caption-preflight327-run.sh; root's copy348 audit records these hashes). 347 (CC1) is cut from it;
# root 354 adopts 347 contingent on the final master's schedule and text equality and caption playback.
NATIVE327 = D/'jobs/csd-caption-preflight327'
NATIVE327_CAPTIONS = {'srt': (NATIVE327/'captions/complex_analysis_visual.en.srt',
                              '5d06423248f088a34774f361721bf05e1c4950ac834086e3666661e1adfe5e1b'),
                      'vtt': (NATIVE327/'captions/complex_analysis_visual.en.vtt',
                              '26e0d7e0eec26eb6f9136bb56f795a6ff8ab9cead26f29fc76a75e9abc4bb2e3')}
DELIVERY347 = {'srt': (Q/'cc1-caption-repair347-candidate.en.srt', '0ea9277e6d598540e5e9057a84ee84daf7f88420b076434659fc3d3ad0ff3230'),
               'vtt': (Q/'cc1-caption-repair347-candidate.en.vtt', '6a383cb7f207be81254e9a820ecb701ca77d00a26557194c675dc1717ef6979b')}
ADOPTION354 = (Q/'root-caption347-review354.json', '749aeba8350d46e339a517af971086bd2d6e2fe039e6501f16e480edd3ad3cb4')
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
# M1 (review 323): root's frame-by-frame window reviews (305) and CC1's 319, read from these records at these hashes
# (Q-relative). Every manifest names preview274-review305/preview/<chapter>.mp4: binary 268's 540p60 renders on the
# timeline269 schedule that 318 requires of every final, so a window frame is the same chapter-local frame.
TRANSITION_SOURCES = {
    'bump-transition-review305.json': 'bc48a64de5fe9c69e00874fed6ba1326959f4fbb4da07409cfed35a4a0e6e2b3',
    'bump-transition-frames305/manifest.json': 'e2341ee728c464a4220aada1459a064d29f48efac7c6cc851c64b9c5753f7f9a',
    'color-transition-review305.json': 'fdd04eab7439fb3a82b230992c67d6b8534b467126b82f059dc9e7ac1ba6f181',
    'color-transition-frames305/manifest.json': 'a58e25460f12f09ed228a1ddd7b5df097048c7d2b27cb746c428a78c9b75e192',
    'gallery319-disposition305.json': 'de33e72ed1f3e7bd10d2e8f41f6e2252fac5fcaa089054861cf93a16c376fac6',
    'cc1-gallery-transition-review319.json': '58a7381dd8506497e10bd147d54837a6472e72651e3724e9dd99747f83e6e651',
    'cc1-gallery-transition-review319.md': 'd0df748083405de1b91924f90db894aaa6b6456493f09c89c12c98e3a9a3e679',
    'gallery-transition-frames305/manifest.json': '5a98dd205a39f9b596f61e9c3dcc048b75c3aa9f1aac286f1872ab26acfe7d49',
    'one-point-transition-review305.json': '02e8594d4b8b92bb4654b115e5f1b13fbc4f1d3c20e14aad3ac943c31ad857ee',
    'one-point-transition-frames305/manifest.json': 'e6fa64dd79ec4f2e2478c65e2ea8b4cec6a1bf12bb82504c31fe7e8bace874a9',
    'open-region-playback-second305.json': '8ede28c42e0117485681c056fedb4b3e66c18a95827bb1026d1d0d3895e1a22c',
    'rigidity-transition-review305.json': 'ac897e91918f60183d0c500b2922375bf5d682317a982c0c68baaf3960032bea',
    'rigidity-transition-frames305/manifest.json': 'ba0eb53d847dd803015545c924b3a951f85be8257736a747772bc9ae39122c26',
    'counting-opening-review305.json': '5e1f55523d92cca846e060276bb4b2635d305fceee8ff516ae290e8277573aec',
    'counting-opening-frames305/manifest.json': '9b889e043ce97bfbb2ce8b72f8306c3dbd3718a29c52b97f4adefa2b702bea8a',
    'counting-transition-review305.json': 'a8552f249637b7348e1367fb0b74708b727be498aa92760eba67995b3bdc91e7',
    'counting-transition-frames305/manifest.json': '9e74486edaaac78d1a4a868f83507a69378f1be9d64ab0456ddd453a34fc64d6',
    'residue-transition-review305.json': 'b3a90d158b6df6453908c95a755a0cb26257673ccbda61e715efdacf4dee397d',
    'residue-transition-frames305/manifest.json': 'c6cca51f4290b32b7e4f2587900cdf1adefde66bf48d032b32f322edd684eea2',
    'continuation-transition-review305.json': '1a92e0f771a65fc385ad07ee453e32dc71e2aec57106210bc6b9c381a2eaa9a8',
    'continuation-transition-frames305/manifest.json': 'a787e39d74c54651b8bbb3a97f0c75e7c863be4ab946c3004bb5aa153e34649b',
}
# (chapter, record, inclusive chapter-local window): the 31 windows, 1,932 frames
TRANSITION_WINDOWS = (
    ('01_the_bump', 'bump-transition-review305.json', (5496, 5531)), ('01_the_bump', 'bump-transition-review305.json', (6273, 6310)),
    ('04_reading_color', 'color-transition-review305.json', (235, 311)), ('04_reading_color', 'color-transition-review305.json', (6742, 6773)),
    ('05_gallery', 'cc1-gallery-transition-review319.json', (34, 98)), ('05_gallery', 'cc1-gallery-transition-review319.json', (1989, 2072)),
    ('05_gallery', 'cc1-gallery-transition-review319.json', (2894, 2977)), ('05_gallery', 'cc1-gallery-transition-review319.json', (4772, 4855)),
    ('05_gallery', 'cc1-gallery-transition-review319.json', (6181, 6264)), ('05_gallery', 'cc1-gallery-transition-review319.json', (7312, 7365)),
    ('05_gallery', 'cc1-gallery-transition-review319.json', (8715, 8859)), ('05_gallery', 'cc1-gallery-transition-review319.json', (9168, 9220)),
    ('05_gallery', 'cc1-gallery-transition-review319.json', (9817, 9878)),
    ('07_one_point', 'one-point-transition-review305.json', (2216, 2280)), ('07_one_point', 'one-point-transition-review305.json', (4744, 4778)),
    ('10_rigidity', 'rigidity-transition-review305.json', (3735, 3799)), ('10_rigidity', 'rigidity-transition-review305.json', (6241, 6335)),
    ('10_rigidity', 'rigidity-transition-review305.json', (7410, 7441)),
    ('11_counting', 'counting-opening-review305.json', (34, 87)), ('11_counting', 'counting-transition-review305.json', (7464, 7493)),
    ('11_counting', 'counting-transition-review305.json', (8294, 8344)),
    ('12_residues', 'residue-transition-review305.json', (2475, 2516)), ('12_residues', 'residue-transition-review305.json', (2536, 2577)),
    ('12_residues', 'residue-transition-review305.json', (2763, 2804)), ('12_residues', 'residue-transition-review305.json', (3025, 3154)),
    ('12_residues', 'residue-transition-review305.json', (3911, 3942)),
    ('14_continuation', 'continuation-transition-review305.json', (34, 98)), ('14_continuation', 'continuation-transition-review305.json', (1223, 1257)),
    ('14_continuation', 'continuation-transition-review305.json', (8876, 8940)), ('14_continuation', 'continuation-transition-review305.json', (10179, 10300)),
    ('14_continuation', 'continuation-transition-review305.json', (11724, 11755)),
)
# 05: each 319 site's measured full-plate frame (cc1-gallery-transition-review319.json measured_curves full_frame;
# C13's six panels share 8859). Root: "static edge contacts must be checked in final4K" (gallery319-disposition305.json).
GALLERY_SITES = {'C1': 95, 'C2': 1989, 'C3': 2065, 'C4': 2894, 'C5': 2974, 'C6': 4772, 'C7': 4852, 'C8': 6181, 'C9': 6261,
                 'C10': 7362, 'C11': 8715, 'C12': 8741, 'C13': 8859, 'C14': 9169, 'C15': 9217, 'C16': 9818, 'C17': 9848}
# 07: root's playback instants (open-region-playback-second305.json observed_seconds x 60, rounded), each +-2 frames
ONE_POINT_INSTANTS = {4858: 80.96, 4872: 81.20, 5099: 84.98, 5355: 89.25, 5681: 94.69}
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


SSD442 = Path('/data/ssd/work/jmev-gallery-recovery440')
WORKER442 = SSD442/'css-gallery-ssd442.py'
WORKER442_SHA = 'ce9baf6a922682ea5792ccbdb3758b68383e7e8e23e2e968a83d9136f9448429'
GALLERY442_JOB = 'css-reviewed-final-05-gallery352-ssd442'


NATIVE608 = {
    '09_boundary': ('native553', 'root-boundary595-materialize.py', 'bc2b3dc320da6ea516f2baad37dcfc248f8d492eec37e84acd3447209be43550', 'csd-native-stage595-09-a1', '119ebe43fbde39e90cb1352656be280b71d52aa736751f0e6f819af6ceb9613d'),
    '10_rigidity': ('native521', 'root-native581-materialize.py', '822047a7f57877378140bb7c81ffd5d89ebd99d47310d17f7d341db6a3b59806', 'csd-native-stage581-10-a1', '5b77a1c8a33a41bab3c3987c9d4f40797bb6a6f1a76bbb57f1ab562ae9960312'),
}
KINDS.update({k: v[0] for k, v in NATIVE608.items()})
REVIEW_BASES.update(native553='sampled_preview', native521='sampled_preview')


def expected_worker451(key, row):
    if key in NATIVE608:
        kind, name, digest, job, binary = NATIVE608[key]
        assert row['kind'] == kind and row['job'] == job and row['binary_sha256'] == binary
        return Q/name, digest, job
    if key == '05_gallery':
        assert row['kind'] == 'combined' and Path(row['host']) == CSS
        assert (row['worker'], row['worker_sha256'], row['job']) == (str(WORKER442), WORKER442_SHA, GALLERY442_JOB)
        return WORKER442, WORKER442_SHA, GALLERY442_JOB
    assert row.get('worker') != str(WORKER442), 'SSD442 worker belongs only to gallery'
    kind = row['kind']
    return WORKERS[kind if kind == 'boundary313' else (kind, Path(row['host']))]


def source_job451(key, row):
    if key in NATIVE608:
        expected_worker451(key, row)
        return D/'jobs'/row['job']
    if key == '05_gallery':
        expected_worker451(key, row)
        return SSD442/'jobs'/GALLERY442_JOB
    assert row.get('worker') != str(WORKER442), 'SSD442 source belongs only to gallery'
    return Path(row['host'])/'jobs'/row['job']


def master350():
    """350's completed job and its own records: registry, kinds, binaries, replays, outputs, hard links; and (363)
    each row's frozen worker, job and review basis, and the sampled intervals each sampled_preview final rests on."""
    receipt, rows = stable(MASTER, MASTER_INPUTS)
    # 350 launch, as 318's: guard ... -- /usr/bin/python3 -I -B <350 resolved> master --job J --inputs I --manifest M
    helper = MASTER_HELPER.resolve()
    command = [str(x) for x in receipt['command']]
    assert command[3:9] == [str(helper), 'master', '--job', MASTER.name, '--inputs', str(MASTER_INPUTS)], command
    assert command[9] == '--manifest' and len(command) == 11, command
    assert rows.get(str(helper)) == sha(helper) == MASTER_HELPER_SHA, 'not frozen master350 or not the one its job bound'
    success(BUILD268/'resource-receipt.json')
    registry = read(BUILD268/'qualification.json')['chapters']
    assert len(registry) == len(set(registry)) == 15 and set(registry) == set(KINDS)
    summary = read(MASTER/'master-summary.json')
    assert (summary['version'], summary['host'], summary['frames'], summary['chapters']) == ('606', 'csd', TOTAL_FRAMES, registry)
    assert summary['consumer_binary_sha256'] == BINARY_268 and summary['narration'] == str(NARRATION)
    assert summary['native_bit_identical_replays'] == 15 and list(summary['replays']) == registry
    manifest = summary['manifest']
    assert manifest['path'] == command[10] and rows.get(manifest['path']) == manifest['sha256'], 'not the manifest 350 bound'
    binary313 = build313()
    # Re-run the pinned master's full provenance gates against its frozen manifest.
    spec = importlib.util.spec_from_file_location('qualified_master606', helper)
    master_module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = master_module
    spec.loader.exec_module(master_module)
    checked_registry, checked_records, checked_files = master_module.gates(Path(manifest['path']))
    assert checked_registry == registry
    for record in checked_records:
        del record['staged']
    assert checked_records == summary['rows'], 'Master summary differs from actual qualified chapter records'
    for path in checked_files:
        assert rows.get(str(path)) == sha(path), 'Rechecked chapter dependency was not frozen by master'
    want = {'combined': BINARY_268, 'retained': BINARY_19, 'boundary313': binary313}
    want.update({v[0]: v[4] for v in NATIVE608.values()})
    by = {r['chapter']: r for r in summary['rows']}
    assert [r['chapter'] for r in summary['rows']] == registry
    for key in registry:
        assert by[key]['kind'] == KINDS[key] and by[key]['binary_sha256'] == want[KINDS[key]], key
        # 363: the frozen 344 and 352 workers and their jobs; the review basis 350 recorded for this kind
        kind = KINDS[key]
        assert by[key]['review_basis'] == REVIEW_BASES[kind], key+': not the review basis 350 records for a '+kind+' row'
        if kind != 'retained':
            worker, digest, job = expected_worker451(key, by[key])
            assert (by[key]['worker'], by[key]['worker_sha256'], by[key]['job']) == \
                (str(worker), digest, job % key.replace('_', '-') if '%s' in job else job), key+': not the frozen 344/352 worker and job'
            assert rows.get(str(worker)) == digest, key+': the master did not bind the frozen worker\'s bytes'
    assert summary['review_bases'] == {k: by[k]['review_basis'] for k in registry}
    assert summary['chapter_binaries'] == {k: by[k]['binary_sha256'] for k in registry}
    assert by[BOUNDARY]['job'] == NATIVE608[BOUNDARY][3] and by[BOUNDARY]['narration'] == str(NARRATION313)
    b = summary['boundary_replay']
    assert (b['chapter'], b['bit_identical'], b['captured_by_binary_sha256'], b['replayed_by_binary_sha256']) == \
        (BOUNDARY, True, NATIVE608[BOUNDARY][4], BINARY_268)
    assert b['capture_check_before_master'] == by[BOUNDARY]['capture_check_before_master']
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
    files.update(checked_files)
    files.update(p for p in MASTER.rglob('*') if p.is_file())
    capture = OUT/'soundtrack'/BOUNDARY/'soundtrack_0.wav'
    assert (b['capture']['path'], b['capture']['source'], b['capture']['sha256']) == (str(capture), str(sources[capture]), digest[capture])
    intervals, playback = {}, {}
    for key in registry:
        row, movie = by[key], sources[OUT/(key+'.mp4')]
        # 350 chapter_gate, as 318's: source = host/'jobs'/job, render/<key>.mp4 (foreign hosts at their original paths after import)
        job = source_job451(key, row)
        assert movie == job/'render'/(key+'.mp4') and digest[OUT/(key+'.mp4')] == row['movie_sha256'], key
        native = key in NATIVE608
        evidence_path = job/('materialized.json' if native else 'render-summary.json')
        render = read(evidence_path)
        if native:
            # Gate607/582 above verified the raw import closure, source/binary,
            # retained audio, independent preview, final samples and exact tables.
            assert (render['chapter'], render['frames'], render['raw_movie_sha256']) == (key, row['frames'], row['movie_sha256'])
        else:
            assert (render['chapter'], render['frames'], render['binary_sha256']) == (key, row['frames'], row['binary_sha256']), key
        if row['review_basis'] == 'sampled_preview':
            # 363: the intervals this final rests on, from its own render summary (350 requires it to carry the permit's
            # review_scope), equal to 350's row; a 352 final rests on no playback instant
            scope = row['review_scope'] if native else render['review_scope']
            if not native:
                assert render['permit_sha256'] == row['permit_sha256'] and row['review_scope'] == scope, key+': not the permit 350 bound' 
            assert (scope['basis'], scope['purpose']) == ('sampled_preview', 'technical_4k_render'), key
            pairs = scope['sampled_intervals']
            assert pairs and row['sampled_intervals'] == pairs, key+': 350 recorded other sampled intervals'
            assert all(type(a) is int and type(b) is int and 0 <= a < b < row['frames'] for a, b in pairs), key
            assert key == BOUNDARY or scope['playback_seconds'] == [], key+': a 352 final rests on no playback instant'
            intervals[key] = [(a, b) for a, b in pairs]
            # 377 (367 M1): the playback instants this final rests on, carried to the picture plan; each centre
            # round(s*60) and 2 frames either side must lie inside the chapter
            seconds = scope['playback_seconds']
            assert type(seconds) is list and all(type(s) in (int, float) and 2 <= round(s*FPS) and round(s*FPS)+2 < row['frames']
                                                 for s in seconds), key+': a playback instant outside the chapter'
            if seconds:
                playback[key] = list(seconds)
        listed = {x['path']: x['sha256'] for x in render['files']}
        for staged in [OUT/(key+s) for s in SUFFIXES] + [OUT/'soundtrack'/key/'soundtrack_0.wav']:
            assert listed.get(str(sources[staged])) == digest[staged], str(sources[staged])
        if key == BOUNDARY and not native:
            assert render['boundary_sha256'] == REPAIR310_SHA and render['build313_qualification_sha256'] == sha(BUILD313/'qualification.json')
        with wave.open(str(OUT/'soundtrack'/key/'soundtrack_0.wav')) as w:
            assert (w.getframerate(), w.getnchannels(), w.getsampwidth(), w.getnframes()) == (48000, 2, 2, row['frames']*800), key
        cues = [x for x in (OUT/(key+'.cuelog.tsv')).read_text().splitlines() if x.strip() and not x.startswith('#')]
        assert summary['replays'][key] == dict(cues=len(cues), samples=row['frames']*800), key
        files.update((evidence_path, job/'resource-receipt.json'))
    assert set(intervals) == {k for k in registry if KINDS[k] != 'retained'}
    return dict(registry=registry, by=by, outputs=outputs, sources=sources, summary=summary,
                binary313=binary313, intervals=intervals, playback=playback, files=files)


def schedules(v, m, script):
    """Every staged table is timeline269's and production timeline21's under its chapter's narration folder;
    every line plays a retained take byte-identical to rebind269's and assembly19's; the cues follow the script."""
    offset, chapters, placements, timelines, files = 0, [], [], {}, set()
    for key in m['registry']:
        row = m['by'][key]
        narration = row['narration']
        assert row['start_frame'] == offset, key+': 350 placed it elsewhere'
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


def delivery347(v, m, script, report, placements, total):
    """363: the delivery captions, validated apart from the master's own captions. The master's native SRT and VTT
    are native327's bytes, so 347, cut from native327, belongs to this master's schedule and text; the 347 SRT/VTT
    pass the same pinned exact check (SRT = VTT, inside the video, text equal to the script whole and cue by cue);
    root 354's adoption record names them and native327 and claims nothing. Returns the record and the files read."""
    success(NATIVE327/'resource-receipt.json')
    files = {NATIVE327/'resource-receipt.json'}
    for kind in ('srt', 'vtt'):
        native, digest = NATIVE327_CAPTIONS[kind]
        assert sha(native) == digest, 'not native327\'s '+kind
        assert m['outputs'][kind].read_bytes() == native.read_bytes(), \
            'the master\'s native '+kind+' is not native327\'s: 347 was not cut from this master\'s captions'
        candidate, digest = DELIVERY347[kind]
        assert candidate == candidate.resolve(strict=True) and sha(candidate) == digest, 'not the 347 '+kind+' root adopted'
        files |= {native, candidate}
    path, digest = ADOPTION354
    assert path == path.resolve(strict=True) and sha(path) == digest, 'not root\'s 354 adoption record'
    adoption = read(path)
    cited = {x['sha256'] for x in adoption['evidence']}
    assert {DELIVERY347['srt'][1], DELIVERY347['vtt'][1], NATIVE327_CAPTIONS['srt'][1]} <= cited, '354 does not cite 347 and native327'
    assert adoption['decision'].startswith('Adopt347 ') and all(adoption[k] is False for k in adoption if k.endswith('_acceptance'))
    files.add(path)
    srt, vtt = DELIVERY347['srt'][0], DELIVERY347['vtt'][0]
    checked = v.check_captions(srt.read_text(), vtt.read_text(), script, report, placements, total)
    return dict(srt=str(srt), srt_sha256=sha(srt), vtt=str(vtt), vtt_sha256=sha(vtt),
                native_srt_and_vtt_equal_native327=True, native327_srt_sha256=NATIVE327_CAPTIONS['srt'][1],
                native327_vtt_sha256=NATIVE327_CAPTIONS['vtt'][1], adoption=dict(path=str(path), sha256=digest),
                exact_check=checked, caption_playback_reviewed=False), files


def film(v):
    """All static gates. Returns what verify and pictures use, plus every file they read."""
    m = master350()
    script_path = WORKSPACE268/SCRIPT
    assert sha(script_path) == sha(WORKSPACE313/SCRIPT)
    script = v.read_script(script_path)
    assert len(script) == 105
    chapters, placements, timelines, total, files = schedules(v, m, script)
    report = read(NARRATION/'report.json')
    captions = v.check_captions(m['outputs']['srt'].read_text(), m['outputs']['vtt'].read_text(), script, report, placements, total)
    delivery, used = delivery347(v, m, script, report, placements, total)       # 363: separate from the native proof
    marks = v.check_phrases(read(NARRATION/'phrase-timings.json'), script, NARRATION)
    files |= m['files'] | used | {script_path, WORKSPACE313/SCRIPT, GUARD, Path(__file__).resolve(), *PINS}
    for folder in (NARRATION, NARRATION313, ORIGINAL_NARRATION):
        files.update(p for p in folder.rglob('*') if p.is_file())
    return dict(m=m, script=script, chapters=chapters, placements=placements, timelines=timelines, total=total,
                captions=captions, delivery347=delivery, phrase_marked_cues=len(marks), files=files)


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


def clean(reason):
    """L3: a reason without local home prefixes. plan182's historical reasons begin with an absolute checkout path
    to Q (944 of its samples); that becomes `Q/`, and any other home path is elided."""
    text = re.sub(r'/\S*/production-repair17/', 'Q/', reason)
    return re.sub(r'/(?:Users|home|private)/\S*', '<local path>', text)


def sheet_label(reason):
    """L3: a sheet label reads the anchor: plan182's `Q/<snapshot> ` prefix is dropped, tabs become ' | ', and
    characters outside Latin-1 (which the default PIL font may not draw) become '?'."""
    text = re.sub(r'^(plan182: )Q/\S+ ', r'\1', clean(reason)).replace('\t', ' | ').replace('−', '-')
    return ''.join(c if ord(c) < 256 else '?' for c in text)


def picture_plan(f, layouts, plan182, preview):
    """Root's 305/319 frame windows and 07 instants, plan182's reviewed samples, current and previous layout anchors
    on both clocks, 09's review312 L2 and preview313 frames, chapter seams, and 10 s periodic frames; native 4K
    frames as a separate small list."""
    by = {c['chapter']: c for c in f['chapters']}
    assert [(c['chapter'], c['start_frame'], c['frames']) for c in plan182['chapters']] == \
        [(c['chapter'], c['start_frame'], c['frames']) for c in f['chapters']], 'plan182 chapters differ from the master'
    samples, natives = {}, {}

    def add(table, key, local, reason):
        c = by[key]
        if 0 <= local < c['frames']:
            x = table.setdefault(c['start_frame']+local, dict(chapter=key, local_frame=local, reasons=[]))
            reason = clean(reason)
            if reason not in x['reasons']:
                x['reasons'].append(reason)

    # M1 first, so a window frame's sheet label names its window: every frame of the 31 windows, one native per
    # window (its middle frame; for 05 the 319 sites' full-plate frames, at least one in each window), and the
    # five 07 instants +-2 frames
    for key, record, (a, b) in TRANSITION_WINDOWS:
        assert 0 <= a <= b < by[key]['frames'], (key, a, b)
        for local in range(a, b+1):
            add(samples, key, local, 'root window %d-%d, every frame (%s)' % (a, b, record))
        if key != '05_gallery':
            add(natives, key, (a+b)//2, 'native 4K, window %d-%d middle (%s)' % (a, b, record))
        else:
            sites = [s for s, frame in GALLERY_SITES.items() if a <= frame <= b]
            assert sites, ('a 05 window without a 319 site', a, b)
    for site, local in GALLERY_SITES.items():
        add(natives, '05_gallery', local, 'native 4K, 319 site %s full plate (cc1-gallery-transition-review319.json)' % site)
    for local, seconds in ONE_POINT_INSTANTS.items():
        for d in range(-2, 3):
            add(samples, '07_one_point', local+d, 'root 07 playback instant %.2f s %+d (open-region-playback-second305.json)' % (seconds, d))
    # 363: M1 on the new review basis: every frame of each sampled interval a sampled_preview final rests on (its
    # render summary's review_scope, equal to 350's row), and a native 4K frame at the middle of each such interval
    # that holds none yet
    for key, pairs in f['m']['intervals'].items():
        start, job = by[key]['start_frame'], f['m']['by'][key]['job']
        for a, b in pairs:
            for local in range(a, b+1):
                add(samples, key, local, 'sampled interval %d-%d, every frame (%s review_scope)' % (a, b, job))
            if not any(start+a <= g <= start+b for g in natives):
                add(natives, key, (a+b)//2, 'native 4K, sampled interval %d-%d middle (%s review_scope)' % (a, b, job))
    # 363: the frames root requires in the native final (root 362; root after review 357)
    for key, record, frames, why in FINAL_FRAME_CHECKS:
        for local in frames:
            assert 0 <= local < by[key]['frames'], (key, local)
            add(samples, key, local, '%s (%s)' % (why, record))
            add(natives, key, local, 'native 4K, %s (%s)' % (why, record))
    # 377 (367 M1): each playback instant a sampled_preview final rests on, as 07's: round(s*60) +-2, native at the centre
    for key, seconds in f['m']['playback'].items():
        job = f['m']['by'][key]['job']
        for s in seconds:
            centre = round(s*FPS)
            assert 2 <= centre and centre+2 < by[key]['frames'], (key, s)
            for d in range(-2, 3):
                add(samples, key, centre+d, 'permit playback instant %.3f s %+d (%s review_scope)' % (s, d, job))
            add(natives, key, centre, 'native 4K, permit playback instant %.3f s (%s review_scope)' % (s, job))
    # 377 (367 L1): root 359's 06 windows and the 08 run, every frame, and a native inside each
    for key, record, (a, b) in REQUIRED_WINDOWS:
        assert 0 <= a <= b < by[key]['frames'], (key, a, b)
        start = by[key]['start_frame']
        for local in range(a, b+1):
            add(samples, key, local, 'required window %d-%d, every frame (%s)' % (a, b, record))
        if not any(start+a <= g <= start+b for g in natives):
            add(natives, key, (a+b)//2, 'native 4K, required window %d-%d middle (%s)' % (a, b, record))
    # 377 (367 L2): both ends of each observation range root added, as overview samples
    for key, record, frames, why in FINAL_ENDPOINT_CHECKS:
        for local in frames:
            assert 0 <= local < by[key]['frames'], (key, local)
            add(samples, key, local, '%s (%s)' % (why, record))
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
    were re-verified by 344 and 350 before the 09 final and the master)."""
    receipt = success(PREVIEW313/'resource-receipt.json')
    assert sha(Q/'review-repair-preview313-inputs.json') == receipt['inputs_sha256']
    summary, sampling = read(PREVIEW313/'summary.json'), read(PREVIEW313/BOUNDARY/'sampling.json')
    assert summary['chapters'][BOUNDARY]['zoom_probe_fails'] == []
    files = {PREVIEW313/'resource-receipt.json', Q/'review-repair-preview313-inputs.json', PREVIEW313/'summary.json',
             PREVIEW313/BOUNDARY/'sampling.json'}
    return dict(summary=summary, sampling=sampling, files=files)


def frame_hashes(path, frames):
    """ffmpeg framemd5 rows `stream, dts, pts, duration, size, hash`, video stream 0 only. L1: the timebase must be
    `#tb 0: 1/60`, the pts exactly 0..frames-1 in order and every duration 1, so a gap, repeat or reorder at a join
    refuses here instead of resting on media_facts' one-frame rounding margin."""
    out, pts, durations, timebases = [], [], [], []
    for line in Path(path).read_text().splitlines():
        if line.startswith('#tb '):
            timebases.append(line.strip())
        if not line.strip() or line.startswith('#'):
            continue
        cells = [x.strip() for x in line.split(',')]
        assert len(cells) == 6 and cells[0] == '0', line
        pts.append(int(cells[2]))
        durations.append(int(cells[3]))
        out.append(cells[5])
    name = Path(path).name
    assert timebases == ['#tb 0: 1/%d' % FPS], '%s: timebase %s, not 1/%d' % (name, timebases, FPS)
    assert len(out) == frames, '%s: %d frames, expected %d' % (name, len(out), frames)
    if pts != list(range(frames)):
        k = next(i for i, p in enumerate(pts) if p != i)
        raise AssertionError('%s: frame %d has pts %d (expected %d): a gap, repeat or reorder' % (name, k, pts[k], k))
    bad = [i for i, d in enumerate(durations) if d != 1]
    assert not bad, '%s: frame %d has duration %d, not 1' % (name, bad[0], durations[bad[0]])
    return out


def ldd_libraries(path):
    """The shared libraries a binary or extension loads, as /usr/bin/ldd resolves them; an unresolved one refuses."""
    output = subprocess.check_output(['/usr/bin/ldd', str(path)], text=True, timeout=60, env=dict(PATH=TOOL_PATH))
    assert 'not found' not in output, 'unresolved library for '+str(path)
    return {Path(m[1]).resolve() for m in (re.search(r'(/\S+)', line) for line in output.splitlines()) if m}


def runtime():
    """L4 (launch-final-verify05.py:62-78): ffmpeg, ffprobe and bash resolved on the guard's PATH and the python3
    running this file, with ffmpeg's first -version line for the records."""
    tools = {}
    for tool in TOOLS:
        found = shutil.which(tool, path=TOOL_PATH)
        assert found, tool+' is not on '+TOOL_PATH
        tools[tool] = Path(found).resolve()
    python = Path(sys.executable).resolve()
    assert python == Path(PYTHON).resolve(), 'run with '+PYTHON
    version = subprocess.check_output([str(tools['ffmpeg']), '-version'], text=True, timeout=60,
                                      env=dict(PATH=TOOL_PATH)).splitlines()[0]
    return {*tools.values(), python}, dict(ffmpeg_version=version, tools={k: str(p) for k, p in tools.items()},
                                           python=str(python))


def pillow_runtime():
    """L4, pictures-after-preview16.py:6-31: draw a sheet label with the default font, round-trip a PNG exactly and a
    JPEG by size, then collect Pillow's files and the native libraries its extensions load."""
    import PIL
    from PIL import Image, ImageDraw, ImageFont
    image = Image.new('RGB', (960, 580), (15, 15, 20))
    ImageDraw.Draw(image).text((8, 545), sheet_label('05_gallery 1.583s f25220 | root window 34-98\tevery frame −'),
                               font=ImageFont.load_default(), fill='white')
    stream = io.BytesIO()
    image.save(stream, format='PNG')
    stream.seek(0)
    with Image.open(stream) as decoded:
        decoded.load()
        assert decoded.size == image.size and decoded.mode == 'RGB' and decoded.tobytes() == image.tobytes()
    assert image.crop((0, 540, 960, 580)).tobytes() != Image.new('RGB', (960, 40), (15, 15, 20)).tobytes(), 'no label pixels'
    stream = io.BytesIO()
    image.save(stream, format='JPEG', quality=94)
    stream.seek(0)
    with Image.open(stream) as decoded:
        decoded.load()
        assert decoded.size == image.size and decoded.mode == 'RGB'
    folder = Path(PIL.__file__).parent
    paths = {p.resolve() for p in folder.rglob('*') if p.is_file() and '__pycache__' not in p.parts}
    libraries = set()
    for extension in sorted(p for p in paths if p.suffix == '.so'):
        libraries |= ldd_libraries(extension)
    assert libraries, 'no native Pillow libraries found'
    return paths | libraries, dict(pillow=PIL.__version__, package=str(folder), png_roundtrip_equal=True, jpeg_roundtrip=True,
                                   label_pixels_present=True)


def growth_upper(samples, natives):
    """Upper estimate of the picture job's growth: the largest 960x540 JPEG (78 KiB) and sheet (897 KiB) measured in
    this lane, raw RGB size for each native PNG, and 64 MiB for plan.json, select.filter and sampling.json."""
    sheets = -(-len(samples)//8)
    return len(samples)*78*1024 + sheets*897*1024 + len(natives)*3840*2160*3 + 64*1024**2


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
    tools, tool_record = runtime()
    require(bound, tools)                        # L4: the decoders and shell used are the bound ones
    f = film(v)
    require(bound, f['files'])
    job, master = VERIFY, f['m']['outputs']['master']
    for name in ('strict-video.log', 'decode-logs', 'framemd5', 'encoded-loudness.log', 'caption-screen.json',
                 'caption-screen347.json', 'verification.json'):
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
        c = next(x for x in f['chapters'] if x['chapter'] == key)
        hashes[key] = frame_hashes(job/'framemd5'/(key+'.framemd5'), c['frames'])
        assert len(hashes[key]) == c['frames'], key
    run(decode+['-i', master, '-map', '0:v:0', '-an', '-f', 'framemd5', '-n', job/'framemd5'/'master.framemd5'], job/'framemd5'/'master.log', 14400)
    whole = frame_hashes(job/'framemd5'/'master.framemd5', f['total'])
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
    # 363: the same pinned screen on the 347 delivery SRT, apart from the native one; root 354 adopted it at
    # none over 20 characters per second and no line over 42
    delivery = f['delivery347']
    subprocess.run([PYTHON, '-I', '-B', str(SCREEN), delivery['srt'], str(f['total']), str(FPS), str(job/'caption-screen347.json')],
                   check=True, capture_output=True, text=True, timeout=600)
    screen347 = read(job/'caption-screen347.json')
    assert screen347['srt_sha256'] == delivery['srt_sha256'] and screen347['caption_count'] == delivery['exact_check']['caption_blocks']
    assert not screen347['over_20_characters_per_second'] and not screen347['lines_over_42_characters'], \
        'the 347 delivery captions are not the ones root adopted at 20 characters per second and 42 per line'
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
                  framemd5_timing=dict(timebase='1/%d' % FPS, pts='0..n-1 in order', duration=1,
                                       checked=['master']+list(f['m']['registry'])),
                  runtime=tool_record, extradata_md5=extradata, encoded_loudness=loudness,
                  caption_screen=dict(caption_count=screen['caption_count'],
                                      over_20_characters_per_second=len(screen['over_20_characters_per_second']),
                                      lines_over_42_characters=len(screen['lines_over_42_characters'])),
                  # 363: what each row's final rests on (350's record), and the 347 delivery captions, kept apart
                  review_bases=f['m']['summary']['review_bases'],
                  delivery_captions_347=dict(delivery, caption_screen=dict(caption_count=screen347['caption_count'],
                                             over_20_characters_per_second=0, lines_over_42_characters=0)),
                  **FLAGS, scope='Technical verification of the 350 master only: decode, frame identity, schedule, '
                  'retained takes, exact native captions, the 347 delivery captions\' exact text and screen, and '
                  'loudness. Pictures, motion, listening, fine sync, caption playback and 4K caption legibility, and '
                  'independent AV acceptance remain root\'s.')
    with (job/'verification.json').open('x') as stream:
        json.dump(result, stream, indent=2)
    print('FINAL VERIFY PASS: 350 master decodes cleanly, equals its 15 staged chapters frame for frame, its native '
          'captions equal the script and native327, and the 347 delivery captions pass the same exact check; every '
          'picture still needs root\'s viewing.', flush=True)
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
    for name, digest in {**TRANSITION_SOURCES, **FINAL_FRAME_SOURCES}.items():   # M1 and 363: the records, at their hashes
        assert (Q/name).is_file() and sha(Q/name) == digest, 'review record changed or missing: '+name
    # 363: root 362 names these 06 frames for the native final, on the preview movie it viewed; 357 covers 11/12/14
    d362, r357 = read(Q/'root-preview06-disposition362.json'), read(Q/'cc1-chapter-picture-review357.json')
    d377, r359 = read(Q/DISPOSITION377), read(Q/'derivative-sheets359-review.json')
    assert (d362['chapter'], d362['movie_sha256'], d362['required_final_checks']['frames']) == \
        ('06_one_derivative', PREVIEW243_MOVIE06, list(FINAL_FRAME_CHECKS[0][2])), 'root 362 names other final frames'
    assert set(r357['chapters']) == {key for key, record, _, _ in FINAL_FRAME_CHECKS[1:]}
    # 377: root's disposition of review 367 holds the 11/12/14 selection and the endpoints, exactly as planned here
    assert d377['review'] == REVIEW367, 'root 377 disposes of another review'
    chosen, ends = {}, {}
    for key, record, frames, _ in FINAL_FRAME_CHECKS:
        if record == DISPOSITION377:
            chosen.setdefault(key, set()).update(frames)
    for key, record, frames, _ in FINAL_ENDPOINT_CHECKS:
        ends.setdefault(key, set()).update(frames)
    assert {k: sorted(v) for k, v in chosen.items()} == d377['required_native_frames'], 'root 377 names other native frames'
    assert {k: sorted(v) for k, v in ends.items()} == d377['additional_overview_endpoints'], 'root 377 names other overview endpoints'
    assert '10200-10380' in ' '.join(d377['requirements']), 'root 377 does not retain the 08 run'
    # 377: root 359's 06 windows, on the preview243 movie it viewed; each required window lies inside one of the
    # sampled intervals its final rests on
    assert (r359['chapter'], r359['movie_sha256']) == ('06_one_derivative', PREVIEW243_MOVIE06)
    assert r359['viewed_intervals'] == [list(w) for _, record, w in REQUIRED_WINDOWS if record == 'derivative-sheets359-review.json'], \
        'root 359 names other 06 windows'
    for key, record, (a, b) in REQUIRED_WINDOWS:
        assert any(x <= a and b <= y for x, y in f['m']['intervals'][key]), \
            '%s: required window %d-%d is not among the sampled intervals its final rests on' % (key, a, b)
    for record in (d362, r357, d377, r359):
        assert all(record[k] is False for k in record if k.endswith('_acceptance')), 'a final-frame record claims an acceptance'
    samples, natives = picture_plan(f, layouts, read(PLAN182), preview)
    assert samples and natives and all(0 <= x['frame'] < f['total'] for x in samples + natives)
    return samples, natives, preview['files'] | {Q/name for name in {**TRANSITION_SOURCES, **FINAL_FRAME_SOURCES}}


def balanced(xs):
    if len(xs) == 1:
        return xs[0]
    k = len(xs)//2
    return '('+balanced(xs[:k])+'+'+balanced(xs[k:])+')'


def part_paths(part):
    return Path(str(PICTURES) % part), Path(str(PICTURES_INPUTS) % part)


def plan_digest(samples, natives):
    """The whole plan's identity: each picture job records it, and the native job requires the overview job's."""
    return hashlib.sha256(json.dumps(dict(samples=samples, natives=natives), sort_keys=True).encode()).hexdigest()


def part_upper(part, samples, natives):
    """M1: one picture job's growth_upper(): the overview job writes every sample and sheet, the native job every
    native PNG. A part over PICTURE_PART_BYTES refuses; nothing is regrouped or dropped."""
    return growth_upper(samples, []) if part == 'overview' else growth_upper([], natives)


def part_done(part, digest):
    """An earlier picture job: finished, its inputs re-hashed (stable), cut from this same plan, and (339, CC0 332)
    every picture and sheet it recorded still at the sha256 it recorded, with no picture or sheet missing or added,
    before anything is bound. Returns its record and every file it wrote, so the next job binds them and its guard
    re-hashes them."""
    job, inputs = part_paths(part)
    _, rows = stable(job, inputs)
    record = read(job/'sampling.json')
    assert record['part'] == part and record['plan_sha256'] == digest, 'the %s job was cut from another plan' % part
    assert all(record[k] is False for k in FLAGS)
    folder, items = ('pictures', record['samples']) if part == 'overview' else ('native4k', record['natives'])
    frames_recorded = {Path(x['path']): x['sha256'] for x in items}
    sheets_recorded = {Path(x['path']): x['sha256'] for x in record['sheets']}
    assert len(frames_recorded) == len(items) and len(sheets_recorded) == len(record['sheets']), 'the %s job recorded a path twice' % part
    assert set(frames_recorded) == set((job/folder).iterdir()), 'the %s pictures are not the ones it recorded' % part
    assert set(sheets_recorded) == set(job.glob('sheet-*')) and len(sheets_recorded) == (-(-len(items)//8) if part == 'overview' else 0), \
        'the %s sheets are not the ones it recorded' % part
    for path, recorded in {**frames_recorded, **sheets_recorded}.items():
        assert sha(path) == recorded, 'changed since the %s job recorded it: %s' % (part, path.name)
    return record, {inputs, *map(Path, rows), *(p for p in job.rglob('*') if p.is_file())}


def strip(xs):
    return [dict(frame=x['frame'], chapter=x['chapter'], local_frame=x['local_frame'], reasons=x['reasons']) for x in xs]


def pictures(part):
    from PIL import Image, ImageDraw
    assert part in PICTURE_PARTS, part
    v = shared()
    job, inputs = part_paths(part)
    bound = admitted(job, inputs, 'pictures')
    tools, tool_record = runtime()
    pillow, pillow_record = pillow_runtime()     # L4: exercised again under the guard, its files the bound ones
    require(bound, tools | pillow)
    f = film(v)
    require(bound, f['files'] | verified(f))
    samples, natives, used = plan_for(f)
    require(bound, used)
    digest, upper = plan_digest(samples, natives), part_upper(part, samples, natives)
    assert upper <= PICTURE_PART_BYTES, (part, upper)
    earlier = []
    for p in PICTURE_PARTS[:PICTURE_PARTS.index(part)]:
        record, files = part_done(p, digest)
        require(bound, files)
        earlier.append(record)
    union = None
    if part == PICTURE_PARTS[-1]:
        # M1: the overview job's samples and this job's natives are exactly the plan, frames and reasons
        (overview,) = earlier
        assert strip(overview['samples']) == strip(samples) and overview['natives'] == [], 'overview samples differ from the plan'
        union = dict(parts=list(PICTURE_PARTS), plan_sha256=digest, samples=len(samples), natives=len(natives),
                     frames_and_reasons_equal_plan=True, overview_sampling_sha256=sha(part_paths('overview')[0]/'sampling.json'))
    xs, folder, ext, size = ((samples, 'pictures', 'jpg', (960, 540)) if part == 'overview' else
                             (natives, 'native4k', 'png', (3840, 2160)))
    master = f['m']['outputs']['master']
    with (job/'plan.json').open('x') as stream:
        json.dump(dict(part=part, plan_sha256=digest, samples=samples, natives=natives), stream, indent=2)
    script = job/'select.filter'
    with script.open('x') as stream:
        stream.write("[0:v]select='"+balanced(['eq(n,%d)' % x['frame'] for x in xs])+"'"+
                     (',scale=960:540' if part == 'overview' else '')+'[picked]')
    (job/folder).mkdir()
    codec = ['-q:v', '2'] if part == 'overview' else ['-c:v', 'png', '-compression_level', '3']
    subprocess.run(['ffmpeg', '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2', '-i', str(master), '-an',
                    '-filter_complex_script', str(script), '-map', '[picked]', '-vsync', '0', *codec,
                    str(job/folder/('frame-%06d.'+ext))], check=True, timeout=14400)
    found = sorted((job/folder).glob('frame-*.'+ext))
    assert len(found) == len(xs), (folder, len(found), len(xs))
    for x, path in zip(xs, found):
        with Image.open(path) as im:
            assert im.size == size, path
        x.update(path=str(path), sha256=sha(path))
    sheets = []
    for k in range(0, len(samples) if part == 'overview' else 0, 8):
        canvas = Image.new('RGB', (1920, 4*565), '#10151d')
        draw = ImageDraw.Draw(canvas)
        for i, x in enumerate(samples[k:k+8]):
            with Image.open(x['path']) as im:
                canvas.paste(im.convert('RGB'), ((i % 2)*960, (i//2)*565))
            more = ' (+%d)' % (len(x['reasons'])-1) if len(x['reasons']) > 1 else ''
            draw.text(((i % 2)*960+5, (i//2)*565+542), '%s %.3fs f%d | %s%s' % (
                x['chapter'], x['local_frame']/FPS, x['frame'], sheet_label(x['reasons'][0])[:80], more), fill='white')
        path = job/('sheet-%04d.jpg' % (k//8+1))
        with path.open('xb') as stream:
            canvas.save(stream, format='JPEG', quality=94)
        sheets.append(dict(path=str(path), sha256=sha(path)))
    for path, digest_ in bound.items():
        assert sha(path) == digest_, path
    with (job/'sampling.json').open('x') as stream:
        json.dump(dict(version=VERSION, part=part, parts=list(PICTURE_PARTS), plan_sha256=digest, growth_upper_bytes=upper,
                       union=union, master_sha256=sha(master), frames=f['total'], chapters=f['chapters'],
                       samples=samples if part == 'overview' else [], natives=natives if part == 'native' else [],
                       sheets=sheets, previous_layout_sources=PREVIOUS_LAYOUT_SOURCES,
                       transition_sources=TRANSITION_SOURCES, transition_windows=TRANSITION_WINDOWS,
                       gallery_sites=GALLERY_SITES, one_point_instants=ONE_POINT_INSTANTS, runtime=tool_record,
                       final_frame_sources=FINAL_FRAME_SOURCES, final_frame_checks=FINAL_FRAME_CHECKS,
                       final_endpoint_checks=FINAL_ENDPOINT_CHECKS, required_windows=REQUIRED_WINDOWS,
                       playback_seconds=f['m']['playback'],
                       pillow=pillow_record, **FLAGS, scope='Pictures of the actual 350 master at 960x540 (sheets) and native 4K PNGs; '
                       'root must view every sheet and native frame. No acceptance is recorded here.'), stream, indent=2)
    if part == 'overview':
        # 339 (CC0 332 C7): this line is written into this job's own folder (review 339 L1), so it is not an anchor by
        # itself: root copies the hash into its ledger outside the folder at overview success, re-hashes sampling.json
        # read-only before the native launch, and compares the ledger with the native launch's printed hash, the native
        # inputs row and union.overview_sampling_sha256 (review 339 N1, section 4)
        print('%d pictures in %d sheets await root\'s viewing; the native job is next. sampling.json sha256 %s' % (
            len(samples), len(sheets), sha(job/'sampling.json')), flush=True)
    else:
        print('%d native 4K frames await root\'s viewing; with the overview job, the plan\'s %d samples and %d natives '
              'are all extracted.' % (len(natives), len(samples), len(natives)), flush=True)
    return 0


def launch(job, inputs, mode, files, need, tail=(), **note):
    assert not job.exists(), 'never duplicate '+job.name
    assert not inputs.exists(), 'preserve any earlier attempt: '+inputs.name
    assert len(os.sched_getaffinity(0)) >= THREADS, 'fewer than 16 CPUs in the inherited affinity'
    tools, tool_record = runtime()                       # L4: the four resolved tools
    pillow, pillow_record = pillow_runtime()             # L4: Pillow draws, encodes and decodes before anything is spent
    files = set(files) | tools | pillow
    free = shutil.disk_usage(D/'jobs').free              # L2, as 318 :807-808: a refusal here spends nothing
    assert free >= need, 'free %d bytes below %d; nothing spent, no %s job launched' % (free, need, mode)
    with inputs.open('x') as stream:
        json.dump({'files': [dict(path=str(p), sha256=sha(p)) for p in sorted(files)]}, stream, indent=2)
    print(json.dumps(dict(job=job.name, inputs=str(inputs), inputs_sha256=sha(inputs), files=len(files),
                          free_bytes=free, free_needed_bytes=need, runtime=tool_record, pillow=pillow_record['pillow'],
                          master_helper_sha256=sha(MASTER_HELPER), master_inputs_sha256=sha(MASTER_INPUTS), **note)), flush=True)
    helper = Path(__file__).resolve()
    subprocess.run([PYTHON, '-I', '-B', str(GUARD), '--phase', 'native', '--job', job.name, '--inputs', str(inputs), '--',
                    '/usr/bin/timeout', '86400', PYTHON, '-I', '-B', str(helper), mode, *tail], check=True)
    success(job/'resource-receipt.json')
    return 0


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=['check', 'launch-verify', 'verify', 'launch-pictures', 'pictures'])
    parser.add_argument('--part', choices=PICTURE_PARTS, help='pictures only: the part this guarded job extracts')
    args = parser.parse_args()
    mode = args.mode
    assert (args.part is not None) == (mode == 'pictures'), '--part goes with pictures only'
    if mode == 'verify':
        return verify(None)
    if mode == 'pictures':
        return pictures(args.part)
    v = shared()
    f = film(v)
    if mode == 'launch-verify':
        return launch(VERIFY, VERIFY_INPUTS, 'verify', f['files'], VERIFY_FREE_BYTES)
    samples, natives, used = plan_for(f)
    digest = plan_digest(samples, natives)
    upper = {p: part_upper(p, samples, natives) for p in PICTURE_PARTS}
    for p in PICTURE_PARTS:
        assert upper[p] <= PICTURE_PART_BYTES, 'the %s job\'s upper estimate %.2f GiB is over %.1f GiB; nothing spent' % (
            p, upper[p]/GIB, PICTURE_PART_BYTES/GIB)
    if mode == 'check':
        print(json.dumps(dict(check='ok', frames=f['total'], chapters=[(c['chapter'], c['kind'], c['frames']) for c in f['chapters']],
                              cues=len(f['placements']), captions=f['captions'], picture_samples=len(samples),
                              native_4k=len(natives), sheets=-(-len(samples)//8), plan_sha256=digest,
                              picture_jobs=[dict(part=p, job=part_paths(p)[0].name, growth_upper_estimate_gib=round(upper[p]/GIB, 2),
                                                 launch_free_needed_gib=round((ADMISSION_BYTES+upper[p])/GIB, 2)) for p in PICTURE_PARTS],
                              picture_job_budget_gib=PICTURE_PART_BYTES/GIB, launch_verify_free_needed_gib=VERIFY_FREE_BYTES/GIB,
                              free_gib_now=round(shutil.disk_usage(D/'jobs').free/GIB, 2))))
        return 0
    part = next((p for p in PICTURE_PARTS if not part_paths(p)[0].exists()), None)
    assert part, 'both picture jobs of 377 already exist; never duplicate'
    files, anchors = f['files'] | verified(f) | used, {}
    for p in PICTURE_PARTS[:PICTURE_PARTS.index(part)]:
        files |= part_done(p, digest)[1]             # refuses unless the earlier job succeeded on this same plan, its outputs as recorded
        anchors[p+'_sampling_sha256'] = sha(part_paths(p)[0]/'sampling.json')   # 339: root compares it with its ledger
    job, inputs = part_paths(part)
    return launch(job, inputs, 'pictures', files, ADMISSION_BYTES+upper[part], ('--part', part), part=part, plan_sha256=digest,
                  **anchors)


if __name__ == '__main__':
    raise SystemExit(main())
