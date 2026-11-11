"""Final 4K successor of combined-master993.py (1955ca32) for the four repaired rows. Prepared only;
nothing here has run (CC3 scratch draft final-master-cc3; root and CC2 review it before any use).

Changes from 993, and nothing else:
  - VERSION and the repaired gate's name and sha256 are unresolved (None); every mode refuses until
    root sets them;
  - the base manifest is master993's own (8da77ed7, the stage970-09-a2 publication) in place of 625:
    its 11 rows other than 01/08/09/11 must be equal, row for row, and keep their existing gates;
  - 01/08/09 must be kind native-combined01 and 11 native-counting02 (REPAIRED). Both kinds go to the
    adopted 992 successor (CC0 cc0-native-master-gate-repaired01.py), pinned by name and sha256, with
    992's interface gate(base,row,frames_of). No boundary313 or native844 row is eligible;
  - launch waits on the stage receipts of both kinds, as it did for native844;
  - master's boundary replay takes the 09 native-combined01 row; capture_check_before_master is the
    repaired gate's, which runs the original313 capture checks unchanged.
M1: each repaired row's review scope comes only from that chapter's actual admission, extraction and
viewing (the repaired gate's admitted_scope()); nothing is carried from master993's intervals.

993's description follows unchanged.

Combined-repair complex master on CSD, version 350. Prepared only; nothing here has run.

Adapted from cc0-combined-master318.py (preserved unchanged) for the renders the master will actually
assemble. 09 comes from CC2's final344 (csd-reviewed-boundary-final344.py) in place of final317. The
other ten changed chapters come from CC2's 352 workers (csd-reviewed-final352.py on CSD, or
css-final-repair352.py on CSS through a CSD import) in place of 303/309. 317's, 303's and 309's
permits had to state actually_reviewed_motion. The actual preview reviews are of sampled and
framewise still pictures (and, for 09, playback instants), not of continuous motion. The 344 and 352
permits therefore state what was reviewed, for a technical 4K render only, in one schema: 344's 21
fields and its sampled_review_scope(). 350 accepts that permit on the 09 row and on the 352 rows, and
never on a retained row:
  - sampled_permit_gate() checks the 09 permit:
      - exactly 344's 21 fields, with no motion boolean and no motion intervals;
      - every acceptance flag exactly false, and no key at any depth naming an acceptance, a release or
        a publication with any other value;
      - a review_scope of basis 'sampled_preview' and purpose 'technical_4k_render', naming
        preview313's 09 movie;
      - sampled intervals made only of preview313's own sampled frames, one covering its measured
        zoom-line exit;
      - root 343's playback instants;
      - final-picture, motion and audio review still open.
    350 makes these checks with its own code. It then runs 344's sampled_review_scope(), with its
    source text pinned by sha256, on the same inputs, and requires the same intervals.
  - Root 341, 343 and 345 and cc3 340 must be in Q at the hashes 344 pins. The permit must bind them,
    and none may claim an acceptance.
  - combined_scope() checks a 352 row's permit (cc2-final352-notes.md section 4):
      - only the two 352 workers, each pinned whole (WORKERS352), with 352's names, chapters, host
        and job;
      - the chapter's preview job is 303's (preview243 for 06/08, preview274 otherwise), and its movie
        is the one its own sampling.json names and the worker pins;
      - the frames the permit's bound records show viewed. frame_basis() reads them with 350's own
        code, in the format 352 states. It must equal the worker's review_basis() exactly, and it
        also refuses an item that views its own record;
      - root evidence views that sampling.json (root's full sampled pass);
      - an independent_review file views a record and names the movie;
      - the common sampled scope on the viewed frames (350's code, then the pinned function);
      - playback_seconds [].
    The independent reviewer is not the reviewer, and the independent files are in Q at their hashes
    (350's code, as for 09).
  - A retained row keeps its historical production19 permit, through 318's permit_gate(), unchanged.
    A permit with a review_scope on a retained row is refused, and so is a historical permit on the
    09 or a 352 row.
  - Final344's and each 352 final's render summary must carry the permit's review_scope and
    independent review, the preview's record and the worker's binding, and the x264 settings read
    from the movie itself (crf 16). For 09 that adds the four records; for 352 it adds the worker's
    review basis.
  - The master summary records each row's review basis and review scope. It claims no
    continuous-motion, picture, audio, fine-sync, caption, release or publication acceptance, and it
    is not written if any key naming an acceptance, a release or a publication is not exactly false.
  - Captions: the master runs binary 268's native `captions all` unchanged. Caption347 is bound by
    root separately, outside this file.
The job, inputs and preflight folder carry 350's name. Everything else is 318's.

318's description, which still applies with final344 in place of final317 and 352 in place of 303/309:

Adapted from cc0-combined-master293.py (preserved unchanged). The provenance is mixed:
  - 09_boundary only: the final317 render (csd-reviewed-boundary-final317.py) on binary 313, from
    build313, rebind313 and timeline313. Kind 'boundary313'. A 09 row of any other kind is refused.
  - The other 10 changed chapters: as in 293, combined-repair renders on binary 268 (csd-reviewed-
    final* such as 303, or CSS css-final-repair* such as 309).
  - 02/03/13/15: as in 293, the retained production19 finals, through import311's CSD imports.
Binary 268 stays the master and caption consumer; nothing is re-rendered. 318 adds these gates:
  - Exact 313 source delta. Build313's source copy equals build268's except boundary.rs,
    dc80c541 -> bda7d931 (repair310). Binary 313's hash is read from build313's own record.
  - All 105 rebind313 WAVs byte-identical to rebind269's and assembly19's. Only the three metadata
    files may differ, and their cue, sentence and generation records equal rebind269's.
  - The complete 09 tables: timeline313 = timeline269 = timeline21 under the named narration
    mappings, and final317's render tables = timeline313's.
  - The 09 replay before the master. Preview313 (binary 313) and preview274 (binary 268) ran the
    same narration-only 540p60 command on the same 09 cue log. Their captures must be
    byte-identical, or nothing launches.
  - The 09 replay in the master. Binary 268 replays final317's cue log and null-tests it against
    the soundtrack that binary 313 captured. Every replay line's cue and sample counts are checked
    against the staged cue log and frames.
  - Independent qualification of 317. The permit's independent reviewer is someone other than its
    reviewer, with files in Q, and the row names a review file that cites worker 317's exact sha256.
  - 307 N7. The files the master and caption delivery contract actually reads are frozen: the
    generation receipts, their per-cue WAV, source and request files, the report's source WAVs,
    the units file and review269 map rebind269 bound, the disposition evidence and
    tools/narration_cache.py.
  - 307 L1/L2. Launch runs that contract read-only on the actual staged tables, then checks free
    space, all before the inputs file that spends the version is written.

Derived from trj-final-master184.py, which supplies the master body:
  - every chapter staged by hard link;
  - `master` and then `captions all` run with the narration and score;
  - all 15 soundtrack replays must be bit-identical;
  - ffprobe gates for 4K60 video and 48 kHz stereo AAC;
  - every staged and bound file is re-hashed.
The launcher replaces the waiters trj-master-after-imports184.py and
trj-master-after-chapters184.py: wait for terminal receipts, freeze the inputs, start one
guarded job.

What 293 changed against 184 (kept in 318, except that 09 is now the boundary313 row above):
  - Host. CSD, with binary review-repair268, the timeline269 schedules, the score21 stems and the
    rebind269 narration (ElevenLabs Frederick: 105 WAVs, byte-identical to assembly19's).
  - Chapter list. Root supplies it as a manifest: one row per registry chapter, in registry order.
  - The 11 changed chapters must be combined-repair renders on binary 268.
  - 02/03/13/15 may instead be the existing production19 finals pinned below. That needs source,
    schedule and audio evidence, which this file checks, plus a named root retention decision.

Modes:
  check   read-only: run every gate and print one JSON line per chapter. Writes nothing.
  launch  Optionally wait for the rows' terminal receipts, then re-run every gate. Run the 307 L1
          delivery-contract preflight in its own new folder, then the 307 L2 free-space check.
          Freeze the inputs with an exclusive create and start the one guarded master job. No retry.
  master  Only inside that guard: the 184 body, with every replay line's counts checked.

This file creates no permit, review, import, render, verification or upload.
"""
import argparse
import sys
import ast
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import wave

VERSION = None                 # UNRESOLVED: root sets the master version (digits); every mode refuses until then
D = Path('/data/projects/jmev-stroke-probe-20261106')        # CSD
Q = D/'qualification01/production-repair17'
TRJ = Path('/data/projects/jmev-render-worker-20261107')
CSS = Path('/data/projects/jmev-css-render-worker-20261107')
HZ2 = Path('/data/projects/jmev-hz2-render-worker-20261107')
JOB = 'csd-combined-master%s' % VERSION
INPUTS = Q/('combined-master%s-inputs.json' % VERSION)
GUARD = D/'guard_csd02.py'
PYTHON = '/usr/bin/python3'
BIN = D/'work/bin/review-repair268'
BINARY_268 = '983954c508e42b90d443ad6ca09f2baba7565a4f609d09d2ea821640534d6ec7'
BINARY_19 = 'f4836eeeb4f2465c0d27ca0497f123f0af6bc0db7bc65096098bb20c5abb0f22'
BUILD268 = D/'jobs/csd-review-repair-build268'
BUILD19 = D/'jobs/csd-production-repair-build19'
SOURCE_COPY_268_SHA = '81d28272a7a23b63199e79e6aa1086f01f131f0ddd685d4ae859d04b1de1934c'
# Binary 268 reads the script and runs the delivery contract from its CARGO_MANIFEST_DIR
# (production19 main.rs:48), the crate inside this workspace. Build19 compiled WORKSPACE19.
WORKSPACE268 = D/'work/review-repair268/workspace'
WORKSPACE19 = D/'work/production-repair19/workspace'
# 318: coherent_narration.py loads narration_cache.py beside it (:25), so the contract reads it too.
CONTRACT = ('explainer_corpus/complex_analysis_visual/script.tsv',
            'tools/narration_delivery.py', 'tools/coherent_narration.py', 'tools/narration_cache.py')
# The cwd of the native master and captions (293 :521); the delivery contract resolves the assembly's
# relative generation-receipt and WAV paths against it (root's preflight 305 and 305b).
REPO07 = D/'work/production-repo07'
UNITS = 'explainer_corpus/complex_analysis_visual/narration_units.json'
TIMELINES = D/'jobs/csd-review-repair-timeline269/timeline-output'
TIMELINE269_INPUTS = Q/'review-repair-timeline269-inputs.json'
BASELINE = D/'jobs/csd-production-timeline21/timeline-output'
ORIGINAL_NARRATION = D/'jobs/csd-production-assembly19/narration'
NARRATION = D/'jobs/csd-review-repair-rebind269/narration'
SCORE = D/'jobs/csd-production-score21'
STEMS = SCORE/'score/stems'
KINDS = ('timeline', 'cues', 'cuelog')
SUFFIXES = ('.mp4', '.timeline.tsv', '.cues.tsv', '.cuelog.tsv', '.layout.tsv')
TOTAL_FRAMES = 133662
SCENE_DIR = 'explainer_corpus/complex_analysis_visual/src/chapters/'
# The chapters combined268 changes (csd-reviewed-final287.py CHAPTERS). They are never retained.
CHANGED = ('01_the_bump', '04_reading_color', '05_gallery', '06_one_derivative', '07_one_point',
           '08_loop_sums', '09_boundary', '10_rigidity', '11_counting', '12_residues',
           '14_continuation')
# The existing finals that may be retained.
#   worker/job/inputs/permit: chapter-plan183.json.
#   permit, movie and frames: each final's render-summary.json (final02-terminal167,
#     final03-terminal115, final13-terminal200, final15-terminal81).
#   scene: build268 source-copy.json, equal to production-source-preserved214/receipt.json.
RETAINED = {
    '02_turning_numbers': dict(
        host=TRJ, worker=TRJ/'qualification01/production19/trj-final-chapter19-02.py',
        job='trj-production19-final-02-turning-numbers32',
        inputs=TRJ/'qualification01/production19/final02-32-inputs.json',
        permit=TRJ/'qualification01/production19/review02-32/final-render-permit32.json',
        permit_sha256='ccdff19cd315f5e4893b59fdc11e84486e8db15ab6f9fd63ae73535bd7dc5ac2',
        movie_sha256='bf0891a409f589bd4fc125c038e03dd027ec334436e25cdc129463c659984f07', frames=5372,
        scene='turning.rs', scene_sha256='24bffa50a914c21cc1457c5056c740a3f25149baf9c140630b2b47e0380b11ec'),
    '03_two_planes': dict(
        host=TRJ, worker=TRJ/'qualification01/production19/trj-final-chapter19-02.py',
        job='trj-production19-final-03-two-planes28',
        inputs=TRJ/'qualification01/production19/final03-28-inputs.json',
        permit=TRJ/'qualification01/production19/review03-28/final-render-permit28.json',
        permit_sha256='107aebb15239c67d66569c5e1217c1343c7490901431ef935ce044962e7c64ca',
        movie_sha256='20353d01ee564ef101ecad80bdfa8ec1db87ac2188b2e4642969b371f2e82cf3', frames=7249,
        scene='two_planes.rs', scene_sha256='0f22d63febc18e05dc4123bedf84eeadf967938bcbc4260cfb0d9e8925ff4cb9'),
    '13_real_integral': dict(
        host=CSS, worker=CSS/'qualification01/css-final-chapter78.py',
        job='css-production19-final-13-real-integral96',
        inputs=CSS/'qualification01/final13-96-inputs.json',
        permit=CSS/'qualification01/review13-96/final-render-permit96.json',
        permit_sha256='e86b5d886b6adf15ff2e682308b19b2284f0c00a6229e4dced6994b70290f8e2',
        movie_sha256='49934034d4611b81155925e667dab463072b21ba0240467447b9ddfb52d32f51', frames=10641,
        scene='real_integral.rs', scene_sha256='83eadb5372812fc540738bbfe9c980ffc1cc35e959be9db9f75906ba425299ae'),
    '15_closing': dict(
        host=HZ2, worker=HZ2/'qualification01/production19/hz2-final-chapter53.py',
        job='hz2-production19-final-15-closing53',
        inputs=HZ2/'qualification01/production19/final15-53-inputs.json',
        permit=HZ2/'qualification01/production19/review15-53/final-render-permit53.json',
        permit_sha256='ec74b68375331470ac18c2c7e5db4fc60bb82a247e2eeead844233c453fecd20',
        movie_sha256='3e2440a8785319cf9477e73303e6391c8a4165c7fbfd3fa6f4ad97e81f874fd4', frames=4379,
        scene='closing.rs', scene_sha256='0c217b6b5daee2681a61049b0918ceda7cc211d377ecd0c5f70b8562dd36568b'),
}
ROW_KEYS = {'chapter', 'kind', 'worker', 'worker_sha256', 'job', 'inputs', 'permit'}
IMPORT_KEYS = {'import_receipt', 'import_inputs'}
# 318: chapter 09 comes only from one final on binary 313. No hash of binary 313 or of any final
# output is written here: binary 313's hash is read from build313's qualification and checked against
# the binary's bytes. The outputs are bound through the final's render summary.
# 350: that final is 344's, which replaces 317 (344 refuses to run beside a 317 09 job). Its worker is
# bound through the row's worker_sha256 and the independent worker review the row names.
BOUNDARY_CHAPTER = '09_boundary'
WORKER344 = Q/'csd-reviewed-boundary-final344.py'
JOB344 = 'csd-reviewed-final-09-boundary344'
# 350: the sampled-preview permit (csd-reviewed-boundary-final344.py PERMIT_KEYS, frozen b8458dbc).
# The key set is pinned by the sha256 of its sorted names, joined by newlines: 21 keys, of which
# six end in '_acceptance'. 344's sampled_review_scope() (:164-208) is pinned by the sha256 of its exact
# source text, which ast.get_source_segment reads from the worker on any Python version.
PERMIT344_KEYS_SHA = 'b051136c8e2fc7d191249e3db9f8689940bcb19f8522bd027e4d03f5a5ff3e76'
PERMIT344_FLAGS = 6
SAMPLED_SCOPE_SOURCE_SHA = '0b9d0ca5f8a3060f041f32898b1b966326db0bf225c97492527ab907194a2672'
MOTION_KEYS = ('actually_reviewed_motion', 'motion_intervals')
SCOPE_KEYS = {'basis', 'purpose', 'preview_movie_sha256', 'sampled_intervals', 'playback_seconds', 'open_reviews'}
OPEN_REVIEWS = {'final_picture', 'motion', 'audio'}
CLAIM_WORDS = ('accept', 'release', 'publication')
# The sampled-preview disposition, at the bytes cc3 and root wrote (344 RECORDS; root copies them into Q
# unchanged). 340 is the independent review; 341, 343 and 345 are root's evidence.
INDEPENDENT340 = 'cc3-preview313-pictures340.md'
PICTURES341 = 'root-preview313-pictures341.json'
PLAYBACK343 = 'root-preview313-playback343.json'
DISPOSITION345 = 'root-preview313-disposition345.json'
DISPOSITION344 = {INDEPENDENT340: '7a0c1f9fd38a62c437b66cf1d356ef3d479008070c9ff9af06a51e1d3b6b9c14',
                  PICTURES341: '7350b30a4bcc595a1fffd31c7670c18c97ef4dac00045f46ec916c225132dca9',
                  PLAYBACK343: 'd9f250ec00007fe0687c7bf2a3155652b30df45956c5dfe53e9922dd0ec67e81',
                  DISPOSITION345: 'cf42fd8ebd43d4ae30fc24dee9826fcc630d7d9f50aeba9a2290bee489a4f08e'}
DISPOSITION345_SCOPE = 'Technical 4K render disposition only; not release acceptance'
# 350: the other ten changed chapters come only from CC2's 352 workers (cc2-final352-notes.md), on CSD
# or on CSS (through a CSD import). Each is pinned whole. Both workers carry 344's sampled_review_scope()
# (SAMPLED_SCOPE_SOURCE_SHA) and one review_basis().
CHAPTERS352 = tuple(c for c in CHANGED if c != BOUNDARY_CHAPTER)
WORKER352 = Q/'csd-reviewed-final352.py'
WORKER352_CSS = CSS/'qualification01/css-final-repair352.py'
WORKERS352 = {WORKER352: '23d55c286aba2fc58b77a5af8dbd539293592daa3a4e0e8584f18f324d868760',
              WORKER352_CSS: '1074051b6063f1cd400de58422f7491d2d011f764884b63c4968b03feaf01a8a'}
# 303's preview of each chapter (352 PREVIEWS): preview243 for 06/08, preview274 for the other eight
PREVIEW243 = D/'jobs/csd-review-repair-preview243'
PREVIEW243_CHAPTERS = ('06_one_derivative', '08_loop_sums')
# 352's frame-record format: the movie hash under the first of these keys present, the frames under
# the first of these lists present
FRAME_HASH_KEYS = ('movie_sha256', 'source_sha256', 'sha256')
FRAME_LIST_KEYS = ('samples', 'frames', 'pictures')
BIN313 = D/'work/bin/review-repair313'
WORKSPACE313 = D/'work/review-repair313/workspace'
BUILD313 = D/'jobs/csd-review-repair-build313'
REBIND313 = D/'jobs/csd-review-repair-rebind313'
NARRATION313 = REBIND313/'narration'
TIMELINE313 = D/'jobs/csd-review-repair-timeline313'
PREVIEW313 = D/'jobs/csd-review-repair-preview313'
PREVIEW274 = D/'jobs/csd-review-repair-preview274'
# 350: the preview313 09 records the sampled scope rests on (344 preview_binding)
PREVIEW313_MOVIE09 = PREVIEW313/'preview/09_boundary.mp4'
PREVIEW313_SAMPLING09 = PREVIEW313/'09_boundary/sampling.json'
PREVIEW313_PROBE09 = PREVIEW313/'09_boundary/zoom-probe.json'
# Each 313 job's guard inputs, as the 313 launchers write them (cc0-repair313-notes.md).
MANIFESTS313 = {'csd-review-repair-build313': Q/'review-repair-build313-inputs.json',
                'csd-review-repair-rebind313': Q/'review-repair-rebind313-inputs.json',
                'csd-review-repair-timeline313': Q/'review-repair-timeline313-inputs.json',
                'csd-review-repair-preview313': Q/'review-repair-preview313-inputs.json'}
BOUNDARY = SCENE_DIR+'boundary.rs'
OLD_BOUNDARY_SHA = 'dc80c541799209f10d02701d18c7fab5a14327000719929e5e2fad0c6fd62f9f'   # build268 source copy
REPAIR310 = Q/'boundary-guide-repair310.rs'
REPAIR310_SHA = 'bda7d931498b23bd3203cd0c6106de69dc0a8c5e160e738d698376322ff39e53'
METADATA = {'external-units.json', 'report.json', 'assembly.json'}
# Preview274's 09 movie and capture as recorded on CSD (313 launcher pin; preview274-transfer305.json).
PREVIEW274_MOVIE09_SHA = 'dba964d2b0730b0e327eb32b9b9bb81496ddb4ec3422f043d356a390f1de3fb4'
PREVIEW274_CAPTURE09_SHA = 'f7f8aa6042a9f05d1b25db4c789f7f6b44cb393445015c057f1e87d9b0c8df51'
EXTERNAL_V2 = 'elevenlabs-ui-receipt-v2'
# 307 L2: the guard admits at 10 GiB free, and 307 N6 puts the master's growth at 1.2-1.7 GB.
# Launch refuses below this before it writes the inputs file, so a refusal here spends nothing.
LAUNCH_FREE_BYTES = 12*1024**3
REPLAY = re.compile(r"^(\S+)\s+replay = fmn's soundtrack, bit for bit \((\d+) cues, (\d+) samples\)$")


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def success(path):
    row = read(path)
    assert row.get('finished') and row.get('exit') == 0, str(path)
    assert row['inputs_sha256'] == row['final_inputs_sha256'], str(path)
    assert not any(row.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), str(path)
    return row


def absolute(path, real=True):
    """An absolute path that exists. real: also free of symlinks (CSD-side files under D).
    Foreign rows keep the exact lexical path their own receipts name. Their bytes are bound by
    hash either way."""
    path = Path(path)
    assert path.is_absolute() and '..' not in path.parts and path.resolve(strict=True), str(path)
    assert not real or path == path.resolve(), 'symlink-free path required: '+str(path)
    return path


def header(path):
    return dict(line.split('\t', 1) for line in Path(path).read_text().splitlines()
                if line.startswith(('chapter\t', 'fps\t', 'frames\t')))


def flag(command, name):
    values = [command[i+1] for i, item in enumerate(command[:-1]) if item == name]
    assert len(values) == 1, name
    return values[0]


def load(path, chapter):
    spec = importlib.util.spec_from_file_location('worker_'+chapter, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def film():
    """Whole-film gates: the 15-chapter registry, all 45 timeline269 tables, score21, narration."""
    files = {BIN, GUARD, TIMELINE269_INPUTS, BUILD268/'qualification.json', BUILD268/'source-copy.json',
             BUILD19/'qualification.json', SCORE/'score21-summary.json'}
    assert sha(BIN) == BINARY_268
    for job in (BUILD268, BUILD19, NARRATION.parent, TIMELINES.parent, BASELINE.parent, SCORE):
        success(job/'resource-receipt.json')
        files.add(job/'resource-receipt.json')
    q268, q19 = read(BUILD268/'qualification.json'), read(BUILD19/'qualification.json')
    assert q268['binary_sha256'] == BINARY_268 and q19['binary_sha256'] == BINARY_19
    registry = q268['chapters']
    assert registry == q19['chapters'] and len(registry) == len(set(registry)) == 15
    assert set(registry) == set(CHANGED) | set(RETAINED) and not set(CHANGED) & set(RETAINED)
    assert sha(TIMELINE269_INPUTS) == read(TIMELINES.parent/'resource-receipt.json')['inputs_sha256']
    frames = {}
    for key in registry:
        for kind in KINDS:
            new, old = TIMELINES/(key+'.'+kind+'.tsv'), BASELINE/(key+'.'+kind+'.tsv')
            expected = old.read_bytes().replace(str(ORIGINAL_NARRATION).encode(), str(NARRATION).encode())
            assert new.read_bytes() == expected, 'timeline269 differs from timeline21: '+str(new)
            files.update((new, old))
        top = header(TIMELINES/(key+'.timeline.tsv'))
        assert (top['chapter'], top['fps']) == (key, '60')
        frames[key] = int(top['frames'])
        assert read(STEMS/'qa'/(key+'.json'))['passed'] is True
    assert sum(frames.values()) == TOTAL_FRAMES
    score = read(SCORE/'score21-summary.json')
    assert score['all_qa_passed'] and score['chapters'] == registry
    for relative, digest in score['files'].items():
        assert sha(SCORE/'score'/relative) == digest, relative
        files.add(SCORE/'score'/relative)
    assert sha(BUILD268/'source-copy.json') == SOURCE_COPY_268_SHA
    copied = read(BUILD268/'source-copy.json')['files']
    for relative in CONTRACT:
        assert sha(WORKSPACE268/relative) == copied[relative], relative
        files.add(WORKSPACE268/relative)
    wavs = sorted(NARRATION.glob('*.wav'))
    assert len(wavs) == 105, 'not the 105 narration WAVs'
    for path in wavs:
        assert sha(path) == sha(ORIGINAL_NARRATION/path.name), 'narration differs from assembly19: '+path.name
        files.add(ORIGINAL_NARRATION/path.name)
    files.update(p for p in NARRATION.rglob('*') if p.is_file())
    files |= contract_reads()
    return registry, frames, files


def contract_reads():
    """307 N7: what binary 268's master and captions delivery contract reads outside the narration
    folder and CONTRACT. It runs with --narration rebind269 (narration_delivery.py:254-347 and
    coherent_narration.py:489-564). Paths are resolved as it resolves them: relative ones against
    the master's cwd REPO07, the map against WORKSPACE268. Each file must be at the hash its own
    record gives. A structure other than the 105 external UI v2 cues refuses here, so nothing is
    skipped."""
    def at(value):
        path = Path(value)
        path = path if path.is_absolute() else REPO07/path
        assert '..' not in path.parts and path.is_file(), 'contract input missing: '+str(path)
        return path

    summary = read(NARRATION.parent/'rebind-summary.json')
    plan = read(NARRATION/'external-units.json')
    assembly, report = read(NARRATION/'assembly.json'), read(NARRATION/'report.json')
    # external_plan() reads the canonical plan, which rebind269 rebound in place
    units_path = WORKSPACE268/UNITS
    assert sha(units_path) == summary['canonical_sha256'], 'workspace268 units differ from rebind269'
    assert read(units_path)['direction_contract'] == plan['direction_contract']
    files = {units_path, NARRATION.parent/'rebind-summary.json'}
    directions = report['meta']['direction_contract']
    source = directions['binding']['contract']['map']
    relative = Path(source['path'])
    assert not relative.is_absolute() and '..' not in relative.parts
    assert relative.is_relative_to(Path(SCENE_DIR).parent.parent)          # the topic tree
    assert sha(WORKSPACE268/relative) == source['sha256'] == summary['map_sha256'], str(relative)
    files.add(WORKSPACE268/relative)
    for item in directions['dispositions']:
        path = at(item['evidence']['path'])
        assert sha(path) == item['evidence']['sha256'], str(path)
        files.add(path)
    units = plan['units']
    records = assembly['manifest']['units']
    assert len(units) == len(records) == 105 and [r['id'] for r in records] == [u['id'] for u in units]
    receipts = {}
    for unit, record in zip(units, records):
        cue, = unit['line_ids']
        wav = at(record['wav'])
        assert sha(wav) == record['sha256'], str(wav)
        receipt_path = at(record['generation_receipt'])
        if receipt_path not in receipts:
            receipts[receipt_path] = read(receipt_path)
        receipt = receipts[receipt_path]
        assert receipt['schema'] == EXTERNAL_V2, str(receipt_path)
        rows = [row for row in receipt['records'] if row['cue'] == cue]
        assert len(rows) == 1, (str(receipt_path), cue)
        row = rows[0]
        assert at(row['wav']).resolve() == wav.resolve() and row['wav_sha256'] == record['sha256'], cue
        found, request = at(row['source']), at(row['request'])
        assert sha(found) == row['source_sha256'] and sha(request) == row['request_sha256'], cue
        files.update((wav, receipt_path, found, request))
    for rows in report['lines'].values():
        for row in rows:
            path = at(row['source'])
            assert sha(path) == row['source_sha256'], str(path)
            files.add(path)
    return files


def permit_gate(chapter, review, binary, table):
    """check()'s permit assertions that hold on any host (287 check, production19 check)."""
    assert review['binary_sha256'] == binary and review['chapter'] == chapter
    assert review['actually_reviewed_pictures'] is True and review['actually_reviewed_motion'] is True
    assert review['unresolved_defects'] == [] and review['reviewer'].strip() and review['observations'].strip()
    assert review['timeline_sha256'] == sha(table)
    assert review['evidence'] and review['motion_intervals']
    for item in review['evidence']:
        assert sha(item['path']) == item['sha256'], item['path']
    frames = review['frames']
    assert type(frames) is int and frames > 0
    for a, b in review['motion_intervals']:
        assert 0 <= a < b < frames
    return {Path(item['path']) for item in review['evidence']}


def claimed(value, at=''):
    """350: every key, at any depth, that names an acceptance, a release or a publication and is not
    exactly False."""
    if type(value) is dict:
        for key, item in value.items():
            if any(word in key.lower() for word in CLAIM_WORDS) and item is not False:
                yield at+key
            yield from claimed(item, at+key+'.')
    elif type(value) is list:
        for n, item in enumerate(value):
            yield from claimed(item, at+str(n)+'.')


def pinned_scope(worker, module):
    """350: 344's sampled_review_scope(), only if its exact source text is the pinned one, the loaded
    function was compiled from this worker file, and none of the names its code uses is redefined
    in the worker's globals (its free names are builtins)."""
    text = Path(worker).read_text()
    nodes = [n for n in ast.parse(text).body if isinstance(n, ast.FunctionDef) and n.name == 'sampled_review_scope']
    assert len(nodes) == 1, 'the worker has no single sampled_review_scope'
    assert hashlib.sha256(ast.get_source_segment(text, nodes[0]).encode()).hexdigest() == SAMPLED_SCOPE_SOURCE_SHA, \
        "sampled_review_scope is not 344's"
    function = module.sampled_review_scope
    codes, names = [function.__code__], set()
    while codes:
        code = codes.pop()
        assert code.co_filename == str(worker), 'sampled_review_scope was not compiled from the worker'
        names.update(code.co_names)
        codes += [c for c in code.co_consts if type(c) is type(code)]
    shadowed = names & set(vars(module))
    assert not shadowed, 'the worker redefines names sampled_review_scope uses: '+', '.join(sorted(shadowed))
    return function


def disposition344(review, movie, sampling):
    """350: root 341, 343 and 345 and cc3 340 in Q at 344's pins, bound by the permit (340 as its
    independent review), naming preview313's 09 movie and sampling, and claiming nothing. Returns root
    343 and the four paths."""
    paths = {name: Q/name for name in DISPOSITION344}
    for name, path in paths.items():
        assert path == path.resolve(strict=True) and sha(path) == DISPOSITION344[name], 'not the recorded disposition file: '+name
    evidence = {item['path']: item['sha256'] for item in review['evidence']}
    independent = {item['path']: item['sha256'] for item in review['independent_review']}
    for name, path in paths.items():
        assert (independent if name == INDEPENDENT340 else evidence).get(str(path)) == DISPOSITION344[name], \
            'the permit must bind '+name
    pictures, playback, verdict = (read(paths[name]) for name in (PICTURES341, PLAYBACK343, DISPOSITION345))
    record = read(sampling)
    text = paths[INDEPENDENT340].read_text()
    assert sha(movie) in text and sha(sampling) in text, 'independent 340 does not name this preview movie and sampling'
    assert pictures['movie_sha256'] == playback['movie_sha256'] == sha(movie) and pictures['sampling_sha256'] == sha(sampling)
    assert pictures['selected_pictures_viewed'] == len(record['samples'])
    assert pictures['sheets_viewed'] == list(range(1, len(record['sheets'])+1))
    assert pictures['concrete_picture_defects_observed'] == []
    assert verdict['scope'] == DISPOSITION345_SCOPE and verdict['independent_review_read_completely'] is True
    cited = {item['sha256'] for item in verdict['evidence']}
    assert {DISPOSITION344[name] for name in (INDEPENDENT340, PICTURES341, PLAYBACK343)} <= cited, \
        'disposition 345 does not cite 340, 341 and 343'
    for name, kept in ((PICTURES341, pictures), (PLAYBACK343, playback), (DISPOSITION345, verdict)):
        found = list(claimed(kept))
        assert not found, name+' claims '+', '.join(found)
    return playback, [paths[name] for name in DISPOSITION344]


def sampled_scope(chapter, review, binary, table, host, preview_job, movie_sha256, sampled, worker, module):
    """350: what every sampled-preview permit must hold (344's schema, which 352 shares), checked
    with 350's own code, then by the worker's pinned sampled_review_scope(), which must return the
    same intervals. sampled: the frames of the reviewed preview movie's own bound pictures.
    Returns the row's review basis (without the chapter-specific records)."""
    keys = set(review)
    assert keys == set(module.PERMIT_KEYS), "not exactly 344's permit fields: %s" % sorted(keys ^ set(module.PERMIT_KEYS))
    assert hashlib.sha256('\n'.join(sorted(keys)).encode()).hexdigest() == PERMIT344_KEYS_SHA, "not 344's permit fields"
    assert not keys & set(MOTION_KEYS), 'a sampled-preview permit states no motion boolean or motion intervals'
    flags = sorted(k for k in keys if k.endswith('_acceptance'))
    assert len(flags) == PERMIT344_FLAGS and all(review[k] is False for k in flags), 'every acceptance flag must be false'
    found = list(claimed(review))
    assert not found, 'the permit claims an acceptance, release or publication: '+', '.join(found)
    assert review['binary_sha256'] == binary and review['chapter'] == chapter
    assert (review['host'], review['preview_job']) == (host, preview_job), (review['host'], review['preview_job'])
    assert review['actually_reviewed_pictures'] is True
    assert review['unresolved_defects'] == [] and review['reviewer'].strip() and review['observations'].strip()
    assert review['timeline_sha256'] == sha(table)
    assert review['evidence']
    for item in review['evidence']:
        assert sha(item['path']) == item['sha256'], item['path']
    frames = review['frames']
    assert type(frames) is int and frames > 0
    sampled = list(sampled)
    assert sampled and len(set(sampled)) == len(sampled) and all(type(x) is int for x in sampled), 'sampled frames'
    scope = review['review_scope']
    assert type(scope) is dict and set(scope) == SCOPE_KEYS, 'review_scope fields'
    assert (scope['basis'], scope['purpose']) == ('sampled_preview', 'technical_4k_render'), 'not a sampled-preview technical 4K scope'
    assert scope['preview_movie_sha256'] == movie_sha256, 'review_scope names another preview movie'
    opened = scope['open_reviews']
    assert type(opened) is list and len(set(opened)) == len(opened) and all(type(x) is str for x in opened)
    assert OPEN_REVIEWS <= set(opened), 'final-picture, motion and audio review must stay open'
    have, intervals, end = set(sampled), [], -1
    assert type(scope['sampled_intervals']) is list and scope['sampled_intervals'], 'no sampled intervals'
    for pair in scope['sampled_intervals']:
        assert type(pair) is list and len(pair) == 2 and all(type(x) is int for x in pair), 'sampled interval %r' % (pair,)
        a, b = pair
        assert end < a < b < frames, 'sampled intervals must be sorted, disjoint and inside the chapter: %r' % (pair,)
        assert set(range(a, b+1)) <= have, 'sampled interval [%d, %d] has unsampled frames' % (a, b)
        intervals.append((a, b))
        end = b
    seconds = scope['playback_seconds']
    assert type(seconds) is list and all(type(x) in (int, float) and 0 <= x < frames/60 for x in seconds), 'playback_seconds'
    assert pinned_scope(worker, module)(review, frames, movie_sha256, have) == intervals, \
        "the worker's sampled_review_scope disagrees with 350's reading"
    return dict(review_basis='sampled_preview', review_scope=scope, sampled_intervals=[list(x) for x in intervals],
                sampled_frames=sum(b-a+1 for a, b in intervals), open_reviews=opened, stated_false=flags)


def sampled_permit_gate(chapter, review, binary, table, worker, module):
    """350: the 09 permit (final344): the common sampled scope on preview313's 09 movie and sampled
    pictures, plus 09's own records: the zoom probe, a sampled interval over the measured zoom-line
    exit, root 341/343/345 and cc3 340, and root 343's playback instants. Returns the row's review
    basis and the files to freeze."""
    movie, sampling, probe = PREVIEW313_MOVIE09, PREVIEW313_SAMPLING09, PREVIEW313_PROBE09
    evidence = {item['path']: item['sha256'] for item in review['evidence']}
    assert evidence.get(str(sampling)) == sha(sampling) and evidence.get(str(probe)) == sha(probe), \
        "the permit must bind preview313's 09 sampling.json and zoom-probe.json"
    record = read(sampling)
    assert record['movie_sha256'] == read(probe)['movie_sha256'] == sha(movie)
    sampled = [x['frame'] for x in record['samples']]
    basis = sampled_scope(chapter, review, binary, table, 'csd', PREVIEW313.name, sha(movie), sampled, worker, module)
    shown = read(PREVIEW313/'summary.json')['chapters'][chapter]
    assert shown['frames'] == review['frames'] and shown['zoom_probe_fails'] == []
    exit_frame = shown['zoom_probe_exit_frame']
    assert type(exit_frame) is int, 'no measured zoom-line exit frame'
    assert any(a <= exit_frame <= b for a, b in basis['sampled_intervals']), \
        'no sampled interval covers the zoom-line exit frame %d' % exit_frame
    playback, records = disposition344(review, movie, sampling)
    assert review['review_scope']['playback_seconds'] == playback['playback_observed_seconds'], \
        "playback_seconds must be root 343's instants"
    basis.update(zoom_exit_frame=exit_frame, disposition=[dict(path=str(p), sha256=sha(p)) for p in records])
    return basis, {Path(item['path']) for item in review['evidence']} | {movie, sampling, probe, PREVIEW313/'summary.json'} | set(records)


def frame_basis(chapter, review, preview_job, movie):
    """350: the frames a 352 permit's bound records show viewed, read with 350's own code. It follows
    the format 352 states (its review_basis() docstring; cc2-final352-notes.md section 4) and returns
    the shape review_basis() returns, so the two can be compared whole.
      - Every file bound in evidence and independent_review is read at the hash the permit gives.
      - A frame record is a bound JSON object whose movie hash is this movie's. The hash is read under
        the first of FRAME_HASH_KEYS present, and the frames from the first of FRAME_LIST_KEYS that
        holds a list.
      - An item's 'views' lists the sha256 of bound frame records of this movie. The item's file must
        contain that sha256 or, for a 'samples' record, the sha256 of every one of its sheets (each
        sheet re-hashed). 350 also refuses an item that views its own record, since a sampling.json
        contains its own sheet hashes and would otherwise cite itself.
      - Sampled frames are the viewed records' frames, and runs are their runs of two or more frames.
        Independent lists the independent_review files that view a record and contain the movie
        hash."""
    movie_sha256 = sha(movie)
    bound = review['evidence'] + review['independent_review']
    digests, data = {}, {}
    for item in bound:
        raw = Path(item['path']).read_bytes()
        assert hashlib.sha256(raw).hexdigest() == item['sha256'], 'not at the hash the permit binds: '+item['path']
        digests[item['path']], data[item['path']] = item['sha256'], raw
    records = {}
    for path, raw in data.items():
        try:
            record = json.loads(raw)
        except ValueError:
            continue
        if type(record) is not dict:
            continue
        key = next((k for k in FRAME_HASH_KEYS if k in record), None)
        name = next((k for k in FRAME_LIST_KEYS if type(record.get(k)) is list), None)
        if key is None or record[key] != movie_sha256 or name is None:
            continue
        assert name == 'frames' or all(type(x) is dict for x in record[name]), 'frame record items: '+path
        frames = [x['frame'] if type(x) is dict else x for x in record[name]]
        assert frames and all(type(n) is int for n in frames), 'frame record without integer frames: '+path
        records[path] = (record, name, frames)
    by_digest = {digests[path]: path for path in records}
    viewers, sheets = {path: set() for path in records}, []
    for item in bound:
        views = item.get('views', [])
        assert type(views) is list and all(type(d) is str for d in views), 'views must list sha256 strings: '+item['path']
        raw = data[item['path']]
        for digest in views:
            assert digest in by_digest, 'views names no bound frame record of '+chapter+': '+digest
            path = by_digest[digest]
            assert path != item['path'], 'a record cannot view itself: '+path
            record, name, _ = records[path]
            if digest.encode() not in raw:
                shown = [(s['path'], s['sha256']) for s in record.get('sheets') or []] if name == 'samples' else []
                assert shown and all(d.encode() in raw for _, d in shown), item['path']+' does not cite the record it views: '+digest
                for sheet, digest_of_sheet in shown:
                    assert sha(sheet) == digest_of_sheet, 'sheet not at its hash: '+sheet
                sheets += [p for p, _ in shown if p not in sheets]
            viewers[path].add(item['path'])
    frame_records, unviewed, sampled = [], [], set()
    for path, (record, name, frames) in records.items():
        if not viewers[path]:
            unviewed.append(path)
            continue
        sampled.update(frames)
        frame_records.append(dict(path=path, sha256=digests[path], frames=len(set(frames)), viewed_by=sorted(viewers[path])))
    independent = sorted(item['path'] for item in review['independent_review']
                         if item.get('views') and movie_sha256.encode() in data[item['path']])
    runs = []
    for n in sorted(sampled):
        if runs and n == runs[-1][1]+1:
            runs[-1][1] = n
        else:
            runs.append([n, n])
    return dict(chapter=chapter, preview_job=preview_job, movie=dict(path=str(movie), sha256=movie_sha256),
                frame_records=frame_records, unviewed=sorted(unviewed), independent=independent,
                sampled=sorted(sampled), runs=[r for r in runs if r[0] < r[1]], sheets=sheets)


def combined_scope(chapter, review, binary, table, worker, module):
    """350: a 352 row's sampled-preview permit, on the chapter's own preview movie:
      - the preview job is 303's for the chapter, and its movie is the one the preview's own
        sampling.json names and the worker pins;
      - 350's frame_basis() equals the worker's review_basis();
      - root evidence views that sampling.json (root's full sampled pass), an independent_review file
        views a record and names the movie, and the viewed frames hold at least one run;
      - the common sampled scope (sampled_scope(), then the worker's pinned sampled_review_scope()) on
        the viewed frames;
      - playback_seconds [], since 352 binds no playback record.
    Returns the row's review basis, the worker's review_basis() and the files to freeze."""
    job = (PREVIEW243 if chapter in PREVIEW243_CHAPTERS else PREVIEW274).name
    assert module.PREVIEWS[chapter]['job'] == job, "the worker's preview job is not 303's: "+module.PREVIEWS[chapter]['job']
    movie, sampling = D/'jobs'/job/'preview'/(chapter+'.mp4'), D/'jobs'/job/chapter/'sampling.json'
    assert sha(movie) == read(sampling)['movie_sha256'] == module.MOVIES[chapter], \
        'not the preview movie its own sampling.json names: '+str(movie)
    own = frame_basis(chapter, review, job, movie)
    worker_basis = module.review_basis(chapter, review)
    assert worker_basis == own, "the worker's review_basis disagrees with 350's reading"
    evidence = {item['path'] for item in review['evidence']}
    assert any(r['path'] == str(sampling) and evidence & set(r['viewed_by']) for r in own['frame_records']), \
        'root evidence must view '+str(sampling)
    assert own['independent'], 'no independent_review file views a frame record of this preview and names its movie'
    assert own['runs'], 'no two consecutive viewed frames: a sampled interval needs a framewise picture record'
    basis = sampled_scope(chapter, review, binary, table, module.HOST, job, own['movie']['sha256'], own['sampled'],
                          worker, module)
    assert review['review_scope']['playback_seconds'] == [], '352 binds no playback record: playback_seconds must be []'
    basis.update(preview_movie=own['movie'], frame_records=own['frame_records'], independent_confirmation=own['independent'])
    files = {movie, sampling} | {Path(item['path']) for item in review['evidence'] + review['independent_review']}
    return basis, worker_basis, files | {Path(p) for p in own['sheets']}


def independent_files(review):
    """344's and 352's independent() rule, checked with 350's own code (CSS's independent() needs the
    CSS alias): the permit's independent reviewer is not its reviewer, and each independent_review
    file is directly in Q at its hash."""
    assert review['independent_reviewer'].strip() and review['independent_reviewer'].strip() != review['reviewer'].strip(), \
        'the independent reviewer must be someone other than the reviewer'
    assert review['independent_review'], 'no independent review'
    files = set()
    for item in review['independent_review']:
        path = absolute(item['path'])
        assert path.parent == Q and sha(path) == item['sha256'], item['path']
        files.add(path)
    return files


def x264_settings(movie):
    """350: the x264 settings string in the movie's first 16 MiB, read as 303 and 344 read it."""
    with Path(movie).open('rb') as stream:
        head = stream.read(16*1024*1024)
    pos = head.find(b'x264 - core')
    assert pos >= 0, 'no x264 settings in '+str(movie)
    return head[pos:pos+5000].split(b'\0', 1)[0].decode('utf8', errors='replace')


def schedule(chapter, render, narration, frames):
    """Timeline269's schedule and audio for one rendered chapter:
    - tables equal timeline269's after the one named narration-folder mapping;
    - every narration file equals rebind269's byte for byte;
    - the score rows are score21's stems at score21's hashes."""
    old, new = str(NARRATION), str(narration)
    files = set()
    for kind in ('timeline', 'cues'):
        expected = (TIMELINES/(chapter+'.'+kind+'.tsv')).read_bytes().replace(old.encode(), new.encode())
        assert (render/(chapter+'.'+kind+'.tsv')).read_bytes() == expected, kind
    assert header(render/(chapter+'.timeline.tsv')) == {'chapter': chapter, 'fps': '60', 'frames': str(frames)}
    lines = [row.split('\t') for row in (render/(chapter+'.timeline.tsv')).read_text().splitlines()
             if row.startswith('line\t')]
    assert lines
    for row in lines:
        path = Path(row[4])
        assert path.parent == narration and sha(path) == sha(NARRATION/path.name), str(path)
        files.update((path, NARRATION/path.name))
    base = (TIMELINES/(chapter+'.cuelog.tsv')).read_text().replace(old, new).splitlines()
    actual = (render/(chapter+'.cuelog.tsv')).read_text().splitlines()
    assert actual[:len(base)] == base
    for row in [x.split('\t') for x in base if x.strip() and not x.startswith('#')]:
        assert Path(row[0]).parent == narration and sha(row[0]) == row[1], row[0]
        files.add(Path(row[0]))
    digests = read(SCORE/'score21-summary.json')['files']
    stems = [x.split('\t') for x in actual[len(base):]]
    assert stems
    for row in stems:
        path = Path(row[0])
        assert path.parent == STEMS and digests.get(str(path.relative_to(SCORE/'score'))) == row[1] == sha(path), row[0]
    return files


def imported(row):
    """A foreign chapter reaches CSD only through a completed guarded CSD import. The files it lists
    sit at their original absolute paths."""
    receipt, manifest = absolute(row['import_receipt']), absolute(row['import_inputs'])
    assert receipt.name == 'resource-receipt.json' and receipt.parent.parent == D/'jobs'
    assert manifest.is_relative_to(D)
    state = success(receipt)
    assert sha(manifest) == state['inputs_sha256']
    files = {receipt, manifest}
    for item in read(manifest)['files']:
        assert sha(item['path']) == item['sha256'], item['path']
        files.add(Path(item['path']))
    return files


def retention(chapter, row, pin):
    """Source evidence for a retained final, plus root's named decision.
    The chapter's scene file is the same bytes in build19's workspace, in build268's pinned source
    copy (outside its changed list) and in build268's workspace."""
    decision = row['retention_decision']
    assert set(decision) == {'path', 'sha256'}
    path = absolute(decision['path'])
    assert path.is_relative_to(Q) and sha(path) == decision['sha256']
    relative = SCENE_DIR+pin['scene']
    copied = read(BUILD268/'source-copy.json')
    assert copied['files'][relative] == pin['scene_sha256'] and relative not in copied['changed']
    for tree in (WORKSPACE19, WORKSPACE268):
        assert sha(tree/relative) == pin['scene_sha256'], str(tree/relative)
    return {path, WORKSPACE19/relative, WORKSPACE268/relative}


def completed313(job):
    """A finished, admitted 313 job whose own guard inputs still hold. The file set includes the
    receipt, the manifest its receipt bound and every row of that manifest; for build313 that carries
    build268's and rebind269's closure."""
    receipt = D/'jobs'/job/'resource-receipt.json'
    state = success(receipt)
    assert state.get('admitted') is True, job
    manifest = MANIFESTS313[job]
    assert sha(manifest) == state['inputs_sha256'], 'not the inputs '+job+' bound'
    rows = read(manifest)['files']
    assert rows and len({row['path'] for row in rows}) == len(rows), str(manifest)
    for row in rows:
        assert sha(row['path']) == row['sha256'], ('bound input changed', str(manifest), row['path'])
    return {receipt, manifest} | {Path(row['path']) for row in rows}


def provenance313(frames):
    """Binary 313, rebind313 and timeline313, read from their completed jobs.
    - Source: exactly build268 plus repair310's boundary.rs.
    - Narration: 105 WAVs byte-identical to rebind269's and assembly19's.
    - Tables: the full 09 schedule equals timeline269's and timeline21's under the named mappings.
    Returns binary 313's hash and the files to freeze."""
    files = set()
    for job in MANIFESTS313:
        files |= completed313(job)
    q313, q268 = read(BUILD313/'qualification.json'), read(BUILD268/'qualification.json')
    copy313, copy268 = read(BUILD313/'source-copy.json'), read(BUILD268/'source-copy.json')
    binary = q313['binary_sha256']
    assert sha(BIN313) == binary and binary not in (BINARY_268, BINARY_19)
    assert q313['base_binary_sha256'] == q268['binary_sha256'] == BINARY_268
    assert q313['tests_passed'] == q268['tests_passed'] and q313['clippy_passed'] is True
    assert q313['fmt_check_passed'] is True and q313['explicit_native_plate_fade_probe_passed'] is True
    assert q313['chapters'] == q268['chapters'] and q313['facts_sha256'] == q268['facts_sha256']
    assert sha(BUILD268/'source-copy.json') == SOURCE_COPY_268_SHA == copy313['base_source_copy_sha256']
    before, after = copy268['files'], copy313['files']
    assert set(before) == set(after), 'build313 source file set differs from build268'
    assert {k for k in before if before[k] != after[k]} == {BOUNDARY}, 'source delta is not boundary.rs alone'
    assert q313['changed'] == copy313['changed'] == [BOUNDARY]
    assert (before[BOUNDARY], after[BOUNDARY], sha(REPAIR310)) == (OLD_BOUNDARY_SHA, REPAIR310_SHA, REPAIR310_SHA)
    for relative in CONTRACT+(BOUNDARY,):
        assert sha(WORKSPACE313/relative) == after[relative], relative
        files.add(WORKSPACE313/relative)
    files.update((BIN313, REPAIR310, BUILD313/'qualification.json', BUILD313/'source-copy.json'))
    # narration: every WAV the same bytes in all three folders, and the audio records unchanged
    summary = read(REBIND313/'rebind-summary.json')
    assert summary['wav_files_byte_identical'] == summary['wav_files_equal_rebind269'] == 105
    names = {p.name for p in NARRATION313.iterdir()}
    assert names == {p.name for p in NARRATION.iterdir()} == {p.name for p in ORIGINAL_NARRATION.iterdir()}
    wavs = sorted(n for n in names if n.endswith('.wav'))
    assert len(wavs) == 105
    for name in wavs:
        assert sha(NARRATION313/name) == sha(NARRATION/name) == sha(ORIGINAL_NARRATION/name), name
    assert {n for n in names if sha(NARRATION313/n) != sha(NARRATION/n)} <= METADATA
    new_report, old_report = read(NARRATION313/'report.json'), read(NARRATION/'report.json')
    new_assembly, old_assembly = read(NARRATION313/'assembly.json'), read(NARRATION/'assembly.json')
    assert new_report['lines'] == old_report['lines'] and new_assembly['outputs'] == old_assembly['outputs']
    assert new_assembly['manifest']['units'] == old_assembly['manifest']['units']
    files.update(NARRATION313/n for n in names)
    files.add(REBIND313/'rebind-summary.json')
    # the complete 09 schedule
    comparison = read(TIMELINE313/'comparison.json')
    assert comparison['frames'] == {BOUNDARY_CHAPTER: frames}
    assert comparison['named_path_mapping'] == {str(NARRATION): str(NARRATION313)}
    assert comparison['production_path_mapping'] == {str(ORIGINAL_NARRATION): str(NARRATION313)}
    rows = {(r['chapter'], r['kind']): r for r in comparison['tables']}
    assert len(rows) == len(comparison['tables']) == len(KINDS)
    for kind in KINDS:
        name = BOUNDARY_CHAPTER+'.'+kind+'.tsv'
        new = (TIMELINE313/'timeline-output'/name).read_bytes()
        assert new == (TIMELINES/name).read_bytes().replace(str(NARRATION).encode(), str(NARRATION313).encode()), \
            'timeline313 differs from timeline269: '+name
        assert new == (BASELINE/name).read_bytes().replace(str(ORIGINAL_NARRATION).encode(), str(NARRATION313).encode()), \
            'timeline313 differs from timeline21: '+name
        assert rows[(BOUNDARY_CHAPTER, kind)]['new_sha256'] == hashlib.sha256(new).hexdigest()
        files.update((TIMELINE313/'timeline-output'/name, TIMELINES/name, BASELINE/name))
    files.add(TIMELINE313/'comparison.json')
    return binary, files


def last_capture(folder):
    captures = sorted(folder.glob('soundtrack_*.wav'), key=lambda p: int(p.stem.split('_')[-1]))
    assert captures and [int(p.stem.split('_')[-1]) for p in captures] == list(range(len(captures))), str(folder)
    return captures[-1]


def capture_check09(frames):
    """Binary 313's mix of 09 against binary 268's, before the master. Preview313 and preview274
    ran the same command (render 09 --res 960x540 --fps 60 --crf 18, no --score;
    review-repair-preview313.py:970, cc3-preview-fade274.py:814). Their 09 cue logs are equal under
    the one narration mapping and hold only narration rows. With equal WAV bytes, any difference
    between the captures is unexplained, so nothing launches."""
    success(PREVIEW274/'resource-receipt.json')
    log313, log274 = PREVIEW313/'preview/09_boundary.cuelog.tsv', PREVIEW274/'preview/09_boundary.cuelog.tsv'
    assert log313.read_bytes() == log274.read_bytes().replace(str(NARRATION).encode(), str(NARRATION313).encode())
    assert log313.read_bytes() == (TIMELINE313/'timeline-output'/log313.name).read_bytes()
    rows = [x.split('\t') for x in log313.read_text().splitlines() if x.strip() and not x.startswith('#')]
    assert rows and all(Path(r[0]).parent == NARRATION313 for r in rows), 'preview cue log is not narration only'
    movie274 = PREVIEW274/'preview/09_boundary.mp4'
    new, old = last_capture(PREVIEW313/'preview/soundtrack/09_boundary'), last_capture(PREVIEW274/'preview/soundtrack/09_boundary')
    assert sha(movie274) == PREVIEW274_MOVIE09_SHA and sha(old) == PREVIEW274_CAPTURE09_SHA
    with wave.open(str(new)) as stream:
        assert (stream.getframerate(), stream.getnchannels(), stream.getsampwidth(),
                stream.getnframes()) == (48000, 2, 2, frames*800)
    assert sha(new) == sha(old), 'binary 313 mixes the 09 cue log differently from binary 268; no master'
    return dict(binary313_capture=dict(path=str(new), sha256=sha(new)),
                binary268_capture=dict(path=str(old), sha256=sha(old)), cue_rows=len(rows),
                equal=True), {PREVIEW274/'resource-receipt.json', log313, log274, movie274, new, old}


def independent344(row, worker, review):
    """Independent qualification of the 09 final: the permit's independent reviewer is not its
    reviewer, with review files in Q at their hashes (344's own independent() rule, which is 317's,
    re-checked here), and the row's worker_review file in Q cites worker 344's exact sha256."""
    files = independent_files(review)
    named = row['worker_review']
    assert set(named) == {'path', 'sha256'}
    path = absolute(named['path'])
    assert path.parent == Q and path != worker and sha(path) == named['sha256']
    assert sha(worker) in path.read_text(), 'worker review does not cite worker 344 by its sha256'
    return files | {path}


SSD442 = Path('/data/ssd/work/jmev-gallery-recovery440')
WORKER442 = SSD442/'css-gallery-ssd442.py'
WORKERS352[WORKER442] = 'ce9baf6a922682ea5792ccbdb3758b68383e7e8e23e2e968a83d9136f9448429'

def combined_source447(worker, key, job):
    if worker == WORKER442:
        assert key == '05_gallery' and job == 'css-reviewed-final-05-gallery352-ssd442'
        return SSD442/'jobs'/job
    assert worker in (WORKER352, WORKER352_CSS)
    host, prefix = (D, 'csd-reviewed-final-') if worker == WORKER352 else (CSS, 'css-reviewed-final-')
    assert job == prefix+key.replace('_', '-')+'352'
    return host/'jobs'/job

# Final 4K successor: the four repaired rows and their kinds (option A), on master993's own manifest.
REPAIRED = {'01_the_bump': 'native-combined01', '08_loop_sums': 'native-combined01',
            '09_boundary': 'native-combined01', '11_counting': 'native-counting02'}
REPAIRED_KINDS = ('native-combined01', 'native-counting02')
BASE_MANIFEST = Q/'csd-native-stage970-09-a2-master-manifest.json'
BASE_MANIFEST_SHA = '8da77ed79e9ccc8a06e703e570c85c44de5cd884a047eaed61355167e397e100'
REPAIRED_GATE_NAME = None      # UNRESOLVED: Q name of the adopted 992 successor (CC0 gate draft)
REPAIRED_GATE_SHA = None       # its sha256 after root fills its pins


def unresolved():
    return [name for name, value in (('VERSION', VERSION), ('REPAIRED_GATE_NAME', REPAIRED_GATE_NAME),
                                     ('REPAIRED_GATE_SHA', REPAIRED_GATE_SHA)) if not value]


def chapter_gate(row, frames_of):
    if row['chapter'] in REPAIRED:
        assert row['kind'] == REPAIRED[row['chapter']], 'Only the repaired native row is eligible for this master'
    if row['kind'] in REPAIRED_KINDS:
        assert not unresolved(), 'unresolved pins: '+', '.join(unresolved())
        native = Q/REPAIRED_GATE_NAME
        assert sha(native) == REPAIRED_GATE_SHA
        return load(native, 'native_repaired').gate(sys.modules[__name__], row, frames_of)
    if row['kind'] == 'native521':
        native = Q/'root-native582-gate.py'
        assert sha(native) == 'a4e9f314d90cd83d2d772469f88956f483e98b9f3f0e5a1579c50b923c2d2f9c'
        return load(native, 'native582').gate(sys.modules[__name__], row, frames_of)
    key, kind = row['chapter'], row['kind']
    worker, inputs, permit = (absolute(row[k], real=False) for k in ('worker', 'inputs', 'permit'))
    job = row['job']
    assert Path(job).name == job and sha(worker) == row['worker_sha256'], key
    if kind == 'retained':
        pin = RETAINED[key]                      # KeyError: only 02/03/13/15 can be retained
        assert set(row) == ROW_KEYS | IMPORT_KEYS | {'retention_decision'}
        assert (worker, job, inputs, permit) == (pin['worker'], pin['job'], pin['inputs'], pin['permit'])
        assert sha(permit) == pin['permit_sha256']
        host, binary, table = pin['host'], BINARY_19, BASELINE/(key+'.timeline.tsv')
    elif kind == 'boundary313':
        # 09 only, only final344, only on CSD; binary 313 from build313's record
        assert key == BOUNDARY_CHAPTER, key+': only 09 comes from binary 313'
        assert set(row) == ROW_KEYS | {'worker_review'}
        assert (worker, job) == (WORKER344, JOB344), (str(worker), job)
        host = D
        binary, files313 = provenance313(frames_of[key])
        table = TIMELINE313/'timeline-output'/(key+'.timeline.tsv')
    else:
        assert kind == 'combined', kind
        assert key != BOUNDARY_CHAPTER, '09 comes only from final344 (kind boundary313)'
        # 350: only CC2's 352 workers, at the bytes 350 pins: on CSD, or on CSS through a CSD import
        assert worker in WORKERS352, 'not a 352 combined-repair chapter worker: '+str(worker)
        assert sha(worker) == WORKERS352[worker], 'not the 352 worker bytes 350 binds: '+str(worker)
        host, pattern = (D, 'csd-reviewed-final-') if worker == WORKER352 else (CSS, 'css-reviewed-final-')
        combined_source447(worker, key, job)
        assert set(row) == ROW_KEYS | (set() if host == D else IMPORT_KEYS)
        binary, table = BINARY_268, TIMELINES/(key+'.timeline.tsv')
    module = load(worker, key)
    if kind == 'boundary313':
        # 344, like 317, has no BINARY_SHA: it reads binary 313's hash from build313, as provenance313 does
        assert (module.VERSION, module.CHAPTERS, module.HOST) == ('344', (key,), 'csd')
        assert (Path(module.BIN), Path(module.NARRATION), Path(module.TIMELINES)) == \
            (BIN313, NARRATION313, TIMELINE313/'timeline-output')
        assert module.job_name(key, module.VERSION) == job
        assert module.RECORDS == DISPOSITION344, "the worker binds another disposition than 344's"
    else:
        assert module.BINARY_SHA == binary
    if kind == 'combined':
        # 350: the 352 worker's own names; CSS's job_name takes the chapter alone
        assert (module.VERSION, module.CHAPTERS, module.HOST) == ('352', CHAPTERS352, 'csd' if host == D else 'css')
        assert (module.job_name(key, module.VERSION) if host == D else module.job_name(key)) == job
        assert set(module.MOVIES) == set(CHAPTERS352) and set(module.PREVIEWS) == set(CHAPTERS352)
        if host == CSS:
            assert permit.parent == module.PERMITS and module.permit_name_ok(key, permit.name), \
                'a CSS permit sits in repair352-permits and is named for its chapter: '+str(permit)
    assert sha(module.BIN) == binary
    root = module.CSS if (kind, host) == ('combined', CSS) else module.ROOT    # 290 names its root CSS
    assert root == host, str(root)
    files = {worker, inputs, permit, module.BIN}
    if kind == 'boundary313':
        files |= files313
    if host != D:
        files |= imported(row)
    source = combined_source447(worker, key, job) if kind == 'combined' else host/'jobs'/job
    receipt = source/'resource-receipt.json'
    state = success(receipt)
    command = [str(item) for item in state['command']]
    assert str(worker) in command
    assert (flag(command, '--chapter'), flag(command, '--job'), flag(command, '--inputs'),
            flag(command, '--permit')) == (key, job, str(inputs), str(permit))
    assert sha(inputs) == state['inputs_sha256']
    frozen = read(inputs)['files']
    bound = {item['path']: item['sha256'] for item in frozen}
    assert len(bound) == len(frozen)
    for path, digest in bound.items():
        assert sha(path) == digest, path
    assert bound.get(str(worker)) == sha(worker) and bound.get(str(permit)) == sha(permit)
    files.update(Path(path) for path in bound)
    files.add(receipt)
    summary_path = source/'render-summary.json'
    summary = read(summary_path)
    files.add(summary_path)
    review = read(permit)
    frames = review['frames']
    assert frames == frames_of[key]
    assert (summary['chapter'], summary['frames'], summary['binary_sha256'], summary['permit_sha256']) == \
        (key, frames, binary, sha(permit))
    if kind == 'boundary313':
        assert (summary['host'], summary['preview_job'], summary['boundary_sha256'],
                summary['build313_qualification_sha256']) == \
            ('csd', PREVIEW313.name, REPAIR310_SHA, sha(BUILD313/'qualification.json'))
    outputs = {item['path']: item['sha256'] for item in summary['files']}
    for path, digest in outputs.items():
        assert sha(path) == digest, path
        files.add(Path(path))
    if kind == 'boundary313':
        # 350: the 09 permit is 344's sampled-preview disposition, never a historical permit
        basis, used = sampled_permit_gate(key, review, binary, table, worker, module)
        files |= used
    elif kind == 'combined':
        # 350: a 352 row's permit is 344's schema on the chapter's own preview
        basis, worker_basis, used = combined_scope(key, review, binary, table, worker, module)
        files |= used
    else:
        # 350: a retained row keeps its historical production19 permit on 318's unchanged gate
        assert 'review_scope' not in review, key+': a sampled-scope permit is accepted only on the 09 (final344) and 352 rows'
        files |= permit_gate(key, review, binary, table)
        basis = dict(review_basis='historical_permit')
    if kind == 'boundary313':
        assert review['source_binding_evidence'] and review.get('host') == 'csd'
        for item in review['source_binding_evidence']:
            assert sha(item['path']) == item['sha256'], item['path']
            files.add(Path(item['path']))
        binding = module.preview_binding(key, review)
        files.update(Path(p) for p in binding)
        files |= independent344(row, worker, review)
        files.update(Path(p) for p in module.independent(review))
        again = module.check(key, permit)                  # 344's own check(), re-run as for 303 and 317
        assert (again[0], again[1], [str(p) for p in again[2]]) == (review, sha(permit), [str(p) for p in binding])
        # 350: final344's summary carries this permit's scope, the four records, preview313's inputs
        # and the same binding
        preview = success(PREVIEW313/'resource-receipt.json')
        assert (summary['review_scope'], summary['independent_review'], summary['preview_binary_sha256'],
                summary['preview_inputs_sha256']) == \
            (review['review_scope'], review['independent_review'], binary, preview['inputs_sha256']), \
            "the summary's review scope, independent review or preview313 record differs from the permit's"
        assert sorted((d['path'], d['sha256']) for d in summary['disposition']) == \
            sorted((d['path'], d['sha256']) for d in basis['disposition']), "the summary's disposition records differ"
        assert summary['preview_binding'] == [dict(path=str(p), sha256=sha(p)) for p in binding], \
            "the summary's preview binding differs from the worker's"
        replay_check, used = capture_check09(frames)
        files |= used
    elif kind == 'combined':
        assert review['source_binding_evidence']
        for item in review['source_binding_evidence']:
            assert sha(item['path']) == item['sha256'], item['path']
            files.add(Path(item['path']))
        binding = module.preview_binding(key, review)
        files.update(binding)
        files |= independent_files(review)
        if host == D:
            # The CSD worker's own check(), re-run here as 184 did. A CSS worker's check() and
            # independent() first assert the CSS-only alias, so that row rests on its job's guarded
            # pre- and post-render check() plus the host-independent gates above and below.
            again = module.check(key, permit)
            assert (again[0], again[1], [str(p) for p in again[2]]) == (review, sha(permit), [str(p) for p in binding])
        # 350: the 352 summary carries this permit's scope and independent review, the worker's review
        # basis, 303's preview record and the same binding
        preview = module.PREVIEWS[key]
        receipt_inputs = success(D/'jobs'/preview['job']/'resource-receipt.json')['inputs_sha256']
        assert (summary['host'], summary['preview_job'], summary['preview_inputs_sha256'], summary['preview_binary_sha256']) == \
            (module.HOST, preview['job'], receipt_inputs, preview['binary_sha256']), "the summary's host or preview record differs"
        assert (summary['review_scope'], summary['independent_review']) == (review['review_scope'], review['independent_review']), \
            "the summary's review scope or independent review differs from the permit's"
        assert summary['review_basis'] == {k: worker_basis[k] for k in ('movie', 'frame_records', 'independent', 'runs')}, \
            "the summary's review basis differs from the worker's"
        assert summary['preview_binding'] == [dict(path=str(p), sha256=sha(p)) for p in binding], \
            "the summary's preview binding differs from the worker's"
        if host == CSS:
            assert (summary['worker352_sha256'], summary['narration']) == (sha(worker), str(module.NARRATION)), \
                "the CSS summary names another worker or narration"
    else:
        assert module.check(key, permit) == (review, sha(permit))
        files |= retention(key, row, pin)
    render = source/'render'
    module.verify_tables(key, render, frames)
    files |= schedule(key, render, Path(module.NARRATION), frames)
    if kind == 'boundary313':
        # final344's own tables are timeline313's: timeline and cues exactly, the cue log as its
        # prefix, then score21's stem rows (checked by schedule() and 344's verify_tables)
        for name in (key+'.timeline.tsv', key+'.cues.tsv'):
            assert (render/name).read_bytes() == (TIMELINE313/'timeline-output'/name).read_bytes(), name
        base = (TIMELINE313/'timeline-output'/(key+'.cuelog.tsv')).read_text().splitlines()
        assert (render/(key+'.cuelog.tsv')).read_text().splitlines()[:len(base)] == base
    movie = render/(key+'.mp4')
    if kind == 'retained':
        assert sha(movie) == pin['movie_sha256'] and frames == pin['frames']
    if kind != 'retained':
        # 350: the encoder settings final344 or 352 recorded are the movie's own, x264 at crf 16 (303's)
        settings = x264_settings(movie)
        assert settings == summary['encoder'] and ' rc=crf' in settings and ' crf=16.0' in settings, \
            'the '+key+' movie is not the recorded crf-16 x264 encode'
    captures = sorted((render/'soundtrack'/key).glob('soundtrack_*.wav'), key=lambda p: int(p.stem.split('_')[-1]))
    assert captures
    capture = captures[-1]
    with wave.open(str(capture)) as stream:
        assert (stream.getframerate(), stream.getnchannels(), stream.getsampwidth(),
                stream.getnframes()) == (48000, 2, 2, frames*800)
    staged = [render/(key+suffix) for suffix in SUFFIXES] + [capture]
    device = (D/'jobs').stat().st_dev
    for path in staged:
        assert outputs.get(str(path)) == sha(path) and path.stat().st_dev == device, str(path)
    record = dict(chapter=key, kind=kind, host=str(host), job=job, worker=str(worker),
                  worker_sha256=sha(worker), permit=str(permit), permit_sha256=sha(permit),
                  binary_sha256=binary, narration=str(module.NARRATION), frames=frames,
                  movie_sha256=sha(movie), staged=[str(p) for p in staged], **basis)
    if kind == 'retained':
        record['retention_decision'] = row['retention_decision']
    if host != D:
        record['import_receipt'] = row['import_receipt']
    if kind == 'boundary313':
        record.update(worker_review=row['worker_review'], independent_reviewer=review['independent_reviewer'],
                      capture_check_before_master=replay_check, consumer_binary_sha256=BINARY_268,
                      source_delta={BOUNDARY: [OLD_BOUNDARY_SHA, REPAIR310_SHA]}, encoder=settings)
    if kind == 'combined':
        record.update(independent_reviewer=review['independent_reviewer'], preview_job=module.PREVIEWS[key]['job'],
                      encoder=settings)
    return record, {Path(p) for p in files}


def gates(manifest):
    assert manifest.parent == Q
    rows = read(manifest)['chapters']
    # Only the four repaired rows may change from master993's own 15-row manifest (8da77ed7).
    retained_manifest = BASE_MANIFEST
    assert sha(retained_manifest) == BASE_MANIFEST_SHA
    retained_rows = read(retained_manifest)['chapters']
    assert len(retained_rows) == len(rows) == 15
    assert [r for r in rows if r['chapter'] not in REPAIRED] == [r for r in retained_rows if r['chapter'] not in REPAIRED]
    registry, frames_of, files = film()
    files.add(retained_manifest)
    assert [row['chapter'] for row in rows] == registry, 'manifest rows must be the registry, in order'
    assert len({row['job'] for row in rows}) == 15
    for row in rows:
        want = REPAIRED.get(row['chapter']) or ('combined' if row['chapter'] in CHANGED else None)
        if row['chapter'] == '10_rigidity' and row['kind'] == 'native521':
            want = 'native521'
        assert want is None or row['kind'] == want, row['chapter']+' must be a '+str(want)+' row'
    assert sum(row['kind'] in REPAIRED_KINDS for row in rows) == len(REPAIRED)
    assert not any(row['kind'] in ('boundary313', 'native844') for row in rows)
    files.update((manifest, Path(__file__).resolve()))
    records, start = [], 0
    for row in rows:
        record, used = chapter_gate(row, frames_of)
        record['start_frame'] = start
        start += record['frames']
        records.append(record)
        files |= used
    assert start == TOTAL_FRAMES
    return registry, records, {Path(p) for p in files}


def check(args):
    manifest = absolute(args.manifest)
    try:
        registry, frames_of, _ = film()
    except Exception as error:                   # read-only report
        print(json.dumps(dict(film='refused', error=repr(error))))
        return 1
    print(json.dumps(dict(film='ok', chapters=len(registry), frames=sum(frames_of.values()))))
    refused = 0
    for row in read(manifest)['chapters']:
        try:
            record, _ = chapter_gate(row, frames_of)
            shown = dict(chapter=row['chapter'], kind=record['kind'], ok=True, binary_sha256=record['binary_sha256'],
                         review_basis=record['review_basis'])
            if record['kind'] == 'boundary313':
                shown['capture_check_before_master'] = record['capture_check_before_master']
                shown.update((k, record[k]) for k in ('sampled_intervals', 'sampled_frames', 'zoom_exit_frame', 'open_reviews'))
            if record['kind'] == 'combined':
                shown.update((k, record[k]) for k in ('host', 'preview_job', 'sampled_intervals', 'sampled_frames',
                                                      'independent_confirmation', 'open_reviews'))
            print(json.dumps(shown))
        except Exception as error:
            refused += 1
            print(json.dumps(dict(chapter=row.get('chapter'), ok=False, error=repr(error))))
    return 1 if refused else 0


def preflight(records):
    """307 L1. Run the workspace268 delivery contract that the native master and captions run first
    (kit cli.rs:390-471) on the actual staged tables, before the inputs file is written. The argv
    is the kit's: registry order, gain '0' (kit GAIN_DB 0.0), --score only for master, cwd REPO07.
    It reads only. It writes one new folder of its own (attempt-numbered; earlier attempts are kept)
    holding symbolic links, under the kit's names, to each row's staged tables, plus preflight.json.
    No native binary runs."""
    base = D/'work/tmp'                          # the jobs' TMPDIR; never created here
    assert base.is_dir(), 'missing '+str(base)
    attempt = next(n for n in range(1, 1000) if not (base/('combined-master%s-preflight-a%d' % (VERSION, n))).exists())
    folder = base/('combined-master%s-preflight-a%d' % (VERSION, attempt))
    folder.mkdir()
    for record in records:
        for src, suffix in zip(record['staged'], SUFFIXES):
            if suffix in ('.timeline.tsv', '.cues.tsv', '.cuelog.tsv'):
                (folder/(record['chapter']+suffix)).symlink_to(src)
    topic = WORKSPACE268/'explainer_corpus/complex_analysis_visual'
    results = {}
    for consumer in ('master', 'captions'):
        argv = [PYTHON, '-I', '-B', str(WORKSPACE268/'tools/narration_delivery.py'), '--topic', str(topic),
                '--narration', str(NARRATION), '--consumer', consumer, '--gain-db', '0']
        for record in records:
            argv += ['--chapter-key', record['chapter']]
        argv += ['--timelines', str(folder)] + (['--score', str(STEMS)] if consumer == 'master' else [])
        run = subprocess.run(argv, cwd=REPO07, env=dict(PATH='/usr/bin:/bin'), capture_output=True, text=True,
                             timeout=3600)
        results[consumer] = dict(argv=argv, exit=run.returncode, stdout=run.stdout, stderr=run.stderr)
    path = folder/'preflight.json'
    with path.open('x') as stream:
        json.dump(dict(cwd=str(REPO07), results=results,
                       scope='Read-only delivery-contract preflight; no native master, caption export, '
                             'audio heard or checked, or AV acceptance.'), stream, indent=2)
    for consumer, result in results.items():
        assert result['exit'] == 0, consumer+' delivery contract refused; no master launched: '+result['stderr'][-1500:]
        shown = json.loads(result['stdout'])
        assert (shown['consumer'], shown['verified_cues'], shown['selected_cues'], shown['plan_mode']) == \
            (consumer, 105, 105, 'external-ui'), consumer
    return path


def launch(args):
    manifest = absolute(args.manifest)
    assert manifest.parent == Q and 0 <= args.wait_hours <= 48
    job = D/'jobs'/JOB
    assert not job.exists() and not INPUTS.exists(), 'master350 job or inputs already exist; never duplicate'
    rows = read(manifest)['chapters']
    # Workers are bound through the manifest's worker_sha256; foreign ones exist only after import.
    pins = {p: sha(p) for p in (Path(__file__).resolve(), manifest, GUARD)}
    # Wait on the receipt a row needs last: its CSD import, else its CSD render.
    waits = [D/'jobs'/r['job']/'resource-receipt.json' if r['kind'] in ('native521',)+REPAIRED_KINDS else
             Path(r['import_receipt']) if 'import_receipt' in r else D/'jobs'/r['job']/'resource-receipt.json' for r in rows]
    deadline = time.monotonic() + args.wait_hours*3600
    while True:
        pending = []
        for receipt in waits:
            if not receipt.exists() or not read(receipt).get('finished'):
                pending.append(str(receipt))
                continue
            success(receipt)                     # a finished failure stops here; no retry
        if not pending:
            break
        assert time.monotonic() < deadline, 'not terminal; no master launched: '+', '.join(pending)
        for path, digest in pins.items():
            assert sha(path) == digest, 'changed while waiting: '+str(path)
        time.sleep(30)
    for path, digest in pins.items():
        assert sha(path) == digest, str(path)
    _, records, files = gates(manifest)
    files.update(pins)
    files.add(preflight(records))                # 307 L1, before anything that spends 350
    free = shutil.disk_usage(D/'jobs').free     # 307 L2
    assert free >= LAUNCH_FREE_BYTES, 'free %d bytes below %d; nothing spent, no master launched' % (free, LAUNCH_FREE_BYTES)
    assert not job.exists()
    with INPUTS.open('x') as stream:
        json.dump({'files': [dict(path=str(p), sha256=sha(p)) for p in sorted(files)]}, stream, indent=2)
    subprocess.run([PYTHON, '-I', '-B', str(GUARD), '--phase', 'native', '--job', JOB, '--inputs', str(INPUTS),
                    '--', PYTHON, '-I', '-B', str(Path(__file__).resolve()), 'master', '--job', JOB,
                    '--inputs', str(INPUTS), '--manifest', str(manifest)], check=True)
    success(job/'resource-receipt.json')
    return 0


def replays_of(text, records, out):
    """Every replay line of the native master log, in registry order. Each must carry its chapter's
    staged cue-log row count (master.rs parse_cue_log: non-empty lines not starting with '#') and
    frames x 800 samples. The kit prints a line only after its null test passes (cli.rs:957-984).
    For 09 that means binary 268 replayed final344's cue log, and the result equals, bit for bit,
    the soundtrack binary 313 captured."""
    lines = [line for line in text.splitlines() if "replay = fmn's soundtrack, bit for bit" in line]
    found = [REPLAY.match(line) for line in lines]
    assert all(found), 'unparsed replay line'
    assert [m.group(1) for m in found] == [r['chapter'] for r in records], 'replay lines are not the registry'
    proof = {}
    for match, record in zip(found, records):
        key = record['chapter']
        rows = [x for x in (out/(key+'.cuelog.tsv')).read_text().splitlines() if x.strip() and not x.startswith('#')]
        expected = (len(rows), record['frames']*800)
        assert (int(match.group(2)), int(match.group(3))) == expected, (key, match.group(0))
        proof[key] = dict(cues=expected[0], samples=expected[1])
    return proof


def master(args):
    """trj-final-master184.py's run(), on CSD, for the rows gates() accepts."""
    assert args.job == JOB and args.inputs == INPUTS
    job = D/'jobs'/JOB
    receipt = read(job/'resource-receipt.json')
    assert receipt['admitted'] and 'exit' not in receipt
    assert receipt['policy']['stop_free_gib'] == 8
    assert receipt['policy']['phase_growth_gib'] == 4
    assert sha(INPUTS) == receipt['inputs_sha256']
    frozen = read(INPUTS)['files']
    bound = {row['path']: row['sha256'] for row in frozen}
    assert len(bound) == len(frozen)
    for path, digest in bound.items():
        assert sha(path) == digest, path

    def require(path):
        assert bound.get(str(path)) == sha(path), str(path)

    manifest = absolute(args.manifest)
    require(Path(__file__).resolve())
    require(manifest)
    registry, records, files = gates(manifest)
    for path in files:
        require(path)
    out = job/'final'
    assert not out.exists()
    out.mkdir()
    links = []

    def link(src, dst):
        require(src)
        dst.parent.mkdir(parents=True, exist_ok=True)
        os.link(src, dst)
        links.append(dict(source=str(src), path=str(dst), sha256=sha(src)))

    total = 0
    for record in records:
        key = record['chapter']
        *tables, capture = [Path(p) for p in record['staged']]
        for src, suffix in zip(tables, SUFFIXES):
            link(src, out/(key+suffix))
        link(capture, out/'soundtrack'/key/'soundtrack_0.wav')
        total += record['frames']
    assert total == TOTAL_FRAMES
    with (job/'staged-inputs.json').open('x') as stream:
        json.dump(links, stream, indent=2)
    os.environ.update(PATH='/usr/bin:/bin', TMPDIR=str(D/'work/tmp'), XDG_CACHE_HOME=str(D/'work/cache'))
    for command, positional in [('master', []), ('captions', ['all'])]:
        argv = [str(BIN), command, *positional, '--narration', str(NARRATION)]
        if command == 'master':
            argv += ['--score', str(STEMS)]
        argv += ['--out', str(out)]
        with (job/(command+'-native.log')).open('x') as log:
            subprocess.run(argv, cwd=REPO07, stdout=log,
                           stderr=subprocess.STDOUT, check=True, timeout=14400)
    replays = replays_of((job/'master-native.log').read_text(), records, out)
    assert list(replays) == registry
    boundary_rows = [r for r in records if r['chapter'] == BOUNDARY_CHAPTER and r['kind'] == REPAIRED[BOUNDARY_CHAPTER]]
    assert len(boundary_rows) == 1, 'Exactly one qualified boundary chapter is required'
    boundary = boundary_rows[0]
    capture = out/'soundtrack'/BOUNDARY_CHAPTER/'soundtrack_0.wav'
    boundary_replay = dict(replays[BOUNDARY_CHAPTER], chapter=BOUNDARY_CHAPTER,
                           capture=dict(path=str(capture), source=boundary['staged'][-1], sha256=sha(capture)),
                           captured_by_binary_sha256=boundary['binary_sha256'],
                           replayed_by_binary_sha256=BINARY_268, bit_identical=True,
                           capture_check_before_master=boundary['capture_check_before_master'])
    outputs = {key: out/('complex_analysis_visual'+suffix) for key, suffix in
               [('master', '_master.mp4'), ('srt', '.en.srt'), ('vtt', '.en.vtt')]}
    probe = json.loads(subprocess.check_output(['ffprobe', '-v', 'error', '-show_streams',
                                                '-of', 'json', str(outputs['master'])]))
    videos = [x for x in probe['streams'] if x['codec_type'] == 'video']
    audios = [x for x in probe['streams'] if x['codec_type'] == 'audio']
    assert len(videos) == len(audios) == 1
    video, audio = videos[0], audios[0]
    assert (video['width'], video['height'], video['avg_frame_rate'],
            int(video['nb_frames'])) == (3840, 2160, '60/1', total)
    assert (audio['sample_rate'], audio['channels'], audio['codec_name']) == ('48000', 2, 'aac')
    assert abs(float(audio['duration']) - total/60) <= 1024/48000 + .001
    assert outputs['srt'].stat().st_size and outputs['vtt'].read_text().startswith('WEBVTT')
    for item in links:
        assert sha(item['source']) == sha(item['path']) == item['sha256']
    for path, digest in bound.items():
        assert sha(path) == digest
    for record in records:
        del record['staged']
    result = dict(chapters=registry, frames=total, native_bit_identical_replays=len(replays),
                  replays=replays, boundary_replay=boundary_replay,
                  chapter_binaries={r['chapter']: r['binary_sha256'] for r in records},
                  outputs={key: dict(path=str(path), sha256=sha(path)) for key, path in outputs.items()},
                  version=VERSION, host='csd', binary_sha256=BINARY_268, consumer_binary_sha256=BINARY_268,
                  narration=str(NARRATION),
                  score=str(STEMS), manifest=dict(path=str(manifest), sha256=sha(manifest)), rows=records,
                  full_decode=False, final_picture_review=False, motion_review=False,
                  audio_heard=False, independent_av_acceptance=False,
                  exact_caption_check=False, publication_accepted=False,
                  # 350: what each row's final rests on; no row's review is continuous motion
                  review_bases={r['chapter']: r['review_basis'] for r in records},
                  continuous_motion_acceptance=False, fine_sync_acceptance=False, final_4k_acceptance=False,
                  release_acceptance=False,
                  scope='Technical combined master only. Chapter finals rest on the review_bases named here: the '
                        'historical production19 permits and sampled-preview dispositions for a technical 4K render. '
                        'No continuous-motion, final-picture, audio, fine-sync, caption, AV, release or publication '
                        'acceptance. No audio heard or checked.')
    found = list(claimed(result))
    assert not found, 'the master summary would claim '+', '.join(found)
    with (job/'master-summary.json').open('x') as stream:
        json.dump(result, stream, indent=2)
    return 0


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    modes = parser.add_subparsers(dest='mode', required=True)
    one = modes.add_parser('check')
    one.add_argument('--manifest', required=True, type=Path)
    one = modes.add_parser('launch')
    one.add_argument('--manifest', required=True, type=Path)
    one.add_argument('--wait-hours', type=float, default=0)
    one = modes.add_parser('master')
    one.add_argument('--job', required=True)
    one.add_argument('--inputs', required=True, type=Path)
    one.add_argument('--manifest', required=True, type=Path)
    args = parser.parse_args()
    assert not unresolved(), 'unresolved pins, refusing: '+', '.join(unresolved())
    assert re.fullmatch(r'[0-9]+', VERSION)
    raise SystemExit({'check': check, 'launch': launch, 'master': master}[args.mode](args))
