"""Prepare 744 (CC0): after build748, rebind the one stale hold record, install 667's runtime tools beside 748's topic and
compare all 15 narrated schedules with timeline666. Two guarded CSD jobs. Checks only.

Prepared only: it has not run on any host. Each phase needs a finished build748 (exit 0, stable inputs, run by the
sealed helper 20ebb008) whose workspace is the exact 722 source. It changes no recording, no phrase mark and no
timing. A successor gets new names, and this file stays as it is.

Phase `prepare` (job csd-apportionment-ch06-prepare744). Any failure stops the job, and there is no retry:
  sources    In 656's copy of not_every_time.rs (d3e88f7b), the 06_1 -> 06_2 prediction hold's marker
             `let q = think_here(&kit, stage)?;` is at line 58, as the map records. In 748's copy (04499441, the 722
             candidate) it occurs once, at line 102. The other 14 holds and the 3 draw.rs helper anchors match 748's
             files exactly as recorded.
  rebind     New files in the job folder, built from the original bytes:
               rebind/  the map, with visual_holds[5].sha256 and .line changed and nothing else, under a new name; the
                        unit plan, with only direction_contract.map changed to name it; 667's four tools.
               narration/  the 93 cue WAVs and phrase-timings.json copied byte for byte from 664's narration;
                        report.json with only meta.units_sha256 and the map binding changed; assembly.json with only
                        manifest.units_sha256 and report_sha256 changed. spans_sha256 still names 566's span file.
  verify     coherent_narration.py verify --require-directions on the job's own copy. It rebuilds every cue's PCM from
             its retained source span and compares it sample for sample.
  install    Into 748's workspace, which the binary reads through its compiled topic path: 667's four tools under
             tools/ at the hashes 667 recorded, the new map beside the old one, and the new unit plan as
             narration_units.json after the original is moved aside to narration_units.before-rebind744.json.
             install744.json records this at once. Nothing is deleted or overwritten. A stop after this point leaves
             the install in place.
  recheck    The same verify on the installed files, then narration_delivery.py --consumer render for all 15 chapters,
             the call the binary makes before each narrated render. 748's workspace must then be at INSTALLED744.
             Last, rebind744.json.

Phase `timeline` (job csd-apportionment-ch06-timeline744), after prepare744 has finished:
  timeline666's render command for each of the 15 chapters (64x36, 60 fps, --lint, CRF 16, 4 threads on CPUs 0-3), with
  apportionment-native748 and the 744 narration, then timeline666's comparison. Each timeline.tsv, cues.tsv and
  cuelog.tsv, with the 744 narration folder's path replaced by 664's, must equal timeline666's file at the hash that
  comparison666.json records, and each chapter's frame count must equal 666's. That is 45 tables and 100,886 frames.
  layout.tsv is recorded against 666, as 666 recorded it, and does not stop the job. A render failure stops the job.
  A table mismatch does not: every chapter runs. comparison744.json is written only if all 45 tables and all frame
  counts are equal. Otherwise the job writes mismatch744.json and exits 1.

Process handling is build 748's, and so 731's, word for word: every step runs in the helper's own group, which the
guard created. On a timeout, an exception inside a step or a stop signal sent to the helper, end_group() ends that
group. That does not include ffmpeg, which the engine starts in a process group of its own; only the timeline renders
reach it. guard_csd02's pinned owned-process-stop02 covers it: root has read it, and it signals every owned
descendant through pidfds under subreaper custody, detached ffmpeg included.

Usage, under guard_csd02 (see handoff745):
  apportionment-ch06-prepare744.py inputs|run prepare --job JOB --build BUILD748_JOB
  apportionment-ch06-prepare744.py inputs|run timeline --job JOB --build BUILD748_JOB --prepare PREPARE744_JOB
`inputs` checks everything it can without the guard and writes only the guard's manifest. `run` is the guard's command.

It generates no narration (current narration: ElevenLabs Frederick Surrey). It does not render a final, and it makes
no reuse decision for any retained render. Equal tables for 06 do not make its frames the old 06's (review 725 F2). It
accepts no picture, motion, audio, fine sync, AV or release result. No audio heard or checked.
"""
import argparse
import ast
import copy
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
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
STOPPER = D/'qualification01/owned-process-stop02.py'      # bound by 667 for 666; bound here the same way
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4, work_cap_gib=56, target_growth_cap_gib=20,
              sample_seconds=10)
JOB_NAMES = dict(build=r'csd-apportionment-ch06-build748(-a[0-9]+)?',
                 prepare=r'csd-apportionment-ch06-prepare744(-a[0-9]+)?',
                 timeline=r'csd-apportionment-ch06-timeline744(-a[0-9]+)?')
BUILD656 = JOBS/'csd-apportionment-build656'
ASSEMBLY664 = JOBS/'csd-apportionment-assembly664'
TIMELINE666 = JOBS/'csd-apportionment-timeline666'
INPUTS666 = Q/'csd-apportionment-timeline666-inputs.json'
RUNTIME667 = Q/'apportionment-runtime667.json'
BUILD_HELPER = Q/'apportionment-ch06-build748.py'
BUILD_SHA = '20ebb00854c16b05fb3a0941ebbc94db6870586d2d8f62b60d1811539439339d'
REL = 'narration_lab/third-wave-20261024/root/five-film-release-20261027'
N664 = ASSEMBLY664/'repo'/REL/'apportionment-picture-assembly566/narration'
STEMS_QA = ASSEMBLY664/'repo'/REL/'apportionment-frederick-score445/score/stems/qa'

TOPIC = 'explainer_corpus/apportionment'
# (files, sha256 of sorted "<sha256>  <path>\n"), as builds 731 and 748 compute them. 748's workspace and crate as it
# builds them (731's pins, same sources), then after this helper's install: 4 tools, the new map and the moved-aside
# unit plan added, the unit plan replaced.
WORKSPACE731 = (95, 'eb860846c3347b0eaafeac3ce72437c3b0abc7704a15596a8944350b4e81d1d4')
CRATE731 = (49, '6a63d40f80e564fc81cae39c1adb72329b7312a3075ee9b1d4cbc25b236e5b1d')
INSTALLED744 = (101, '68380f56c5f7fdeeb2583010dcaefe28eb2721378e681f33434c67a62257ae26')
CRATE744 = (51, '037f9166bb6329c7f77e519028040bc14c2604c491a80ea0843aea13b4ead7f2')
# The four 722 candidates as 748 builds them (path under TOPIC: sha256).
CANDIDATES722 = {'src/chapters/not_every_time.rs': '04499441ccde5be5f827b9768da26bbd0672f21e283728b298809f827d7225e9',
                 'src/facts.rs': 'e9b86e26f7e285ddc397a8948462492c824e77a637a4904e501466a7ece78f1d',
                 'oracle.py': '8c1dd3af893059601f1c6a195a3292e7274d796edd4701adfefd6fb3d780c94f',
                 'docs/storyboard.md': '17a46b19463c07dd4713527574bcb46d56e8dc4dba6b648cc1daac4817f8ffbd'}
SCRIPT_SHA = 'b9344e1123170e7e3898d8aea2123e0b659bcd571085008fc97894e7d69da624'
MAP = TOPIC + '/docs/narration-direction-map-frederick01.json'
MAP_SHA = 'd6d21eb2ad95a3c8d25beefd042a06e2c0f2f7fbb9779742ba1cef98734f1f1b'
NEW_MAP = TOPIC + '/docs/narration-direction-map-frederick-rebind744.json'
NEW_MAP_SHA = '38209347ef411d0c343342a1b264f3b714cfb4bb059805250a15888c8b89e587'
UNITS = TOPIC + '/narration_units.json'
UNITS_SHA = 'd18c1f93ff069d9230dd19089c9880fddb82f1bbb089f4e25b8cf6eb70dad80a'
NEW_UNITS_SHA = '9bf31d880c03b7362f34d2f21ddaa4546cc32bd86372c58130932a37b511d8a3'
UNITS_ASIDE = TOPIC + '/narration_units.before-rebind744.json'
HOLD_INDEX = 5
HOLD = dict(video='apportionment', cue='06_1', next_cue='06_2', kind='prediction',
            path=TOPIC + '/src/chapters/not_every_time.rs', marker='let q = think_here(&kit, stage)?;')
OLD_SOURCE = ('d3e88f7bcad146b9078ddc7d84588f6569339cd59cd3b2c225aa963bfadf7c94', 58)
NEW_SOURCE = (CANDIDATES722['src/chapters/not_every_time.rs'], 102)
# 667's four runtime tools, in 667's order; export 652 has the same bytes.
TOOLS = {'coherent_narration.py': '95615cfadb42fbfdddf828340b74ec66d89c975a1f399593e212bdd9840e65d5',
         'narration_cache.py': '47331efd3952af6d0e66971a1c8fddd08929f881d1f9b470bde223db7a01b0c0',
         'narration_delivery.py': '961f17cc85dd30eadca73f950ec0bab6e52eb47ad43a7384c3710c97962b1878',
         'storyboard.py': '3faed8c1ded451c432eddfcdd8d2034a7244594f3f99d753a63dc47e487cb0b7'}
PHRASE_SHA = 'fc3c86fa1b0e8f364ae09c06747ef21f08ac7bbf7fe1d5e666550501a7bc2542'    # provisional phrase marks, 566 = 664
# The only leaves each rebound file may change.
CHANGES = dict(map=['visual_holds/5/line', 'visual_holds/5/sha256'],
               units=['direction_contract/map/path', 'direction_contract/map/sha256'],
               report=['meta/direction_contract/binding/contract/map/path',
                       'meta/direction_contract/binding/contract/map/sha256', 'meta/units_sha256'],
               assembly=['manifest/units_sha256', 'report_sha256'])
CUES = 93
CHAPTERS = ['01_one_more_seat', '02_shares', '03_largest_tails', '04_sixteenth', '05_dials', '06_not_every_time',
            '07_yardstick', '08_signposts', '09_cost', '10_rounding_rules', '11_webster_quota', '12_both_promises',
            '13_population', '14_no_rule', '15_recap']
NEW_CHAPTER = '06_not_every_time'
TABLES = ('timeline.tsv', 'cues.tsv', 'cuelog.tsv')
FRAMES666 = 100886
VERIFY_SECONDS = 1800                                  # 664's verify timeout
TIMELINE_SECONDS = 14400                               # 666's deadline for all 15 renders
STEP_SECONDS = 600
TERM_GRACE = 6                                         # the guard waits 20 s after its SIGTERM, then SIGKILL
STOP_SIGNALS = (signal.SIGTERM, signal.SIGINT, signal.SIGHUP)
HELPER = Path(__file__).resolve()
STATE = dict(job=None, step=None, child=None)          # for end_group()'s record only; stopping never needs it
TIMES = {}
MISSING = object()


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8'))


def map_listing(rows):
    """The listing hash of a {relative path: sha256} map, as listing() computes it for a folder."""
    text = ''.join('%s  %s\n' % (rows[rel], rel) for rel in sorted(rows))
    return len(rows), hashlib.sha256(text.encode()).hexdigest()


def tree_rows(top):
    """{relative path: sha256} of every file; no links, no specials. CC1's and CC0's 492/506 rule."""
    rows = {}
    for parent, dirs, files in os.walk(top, followlinks=False):
        for name in dirs + files:
            path = Path(parent, name)
            if path.is_symlink():
                raise SystemExit('744 refused: symlink in the tree: %s' % path)
        for name in files:
            path = Path(parent, name)
            if not path.is_file():
                raise SystemExit('744 refused: not a regular file: %s' % path)
            rows[path.relative_to(top).as_posix()] = sha(path)
    return rows


def listing(top):
    return map_listing(tree_rows(top))


def check_files(rows):
    for row in rows:
        assert sha(row['path']) == row['sha256'], ('input changed', row['path'])


def success(job, label):
    """A finished guarded job: exit 0, the same inputs at its end as at its start, no stop or error."""
    r = read(job/'resource-receipt.json')
    assert r.get('admitted') is True and r.get('finished') and r.get('exit') == 0, (label + ' receipt', r.get('exit'))
    assert r.get('inputs_sha256') == r.get('final_inputs_sha256'), label + ' inputs changed during its run'
    assert not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), label + ' receipt'
    return r


def manifest_rows(path):
    return {row['path']: row['sha256'] for row in read(path)['files']}


def script_ids(crate):
    ids = []
    for raw in (crate/'script.tsv').read_text(encoding='utf-8').split('\n'):
        line = raw.rstrip('\r')
        if line.strip() and not line.startswith('#'):
            ids.append(line.split('\t')[0].strip())
    assert len(ids) == CUES == len(set(ids)), 'script.tsv does not hold the 93 cues'
    return ids


def strings(value):
    if isinstance(value, dict):
        for v in value.values():
            yield from strings(v)
    elif isinstance(value, list):
        for v in value:
            yield from strings(v)
    elif isinstance(value, str):
        yield value


def closure(starts):
    """Every file under D that these JSON files name, and that any JSON file they name names in turn, at any depth.
    coherent_narration reads receipts and source WAVs from this set; binding it keeps them fixed for the run."""
    found, todo, seen = set(), [Path(p) for p in starts], set()
    while todo:
        path = todo.pop()
        if path in seen:
            continue
        seen.add(path)
        try:
            data = read(path)
        except (UnicodeDecodeError, json.JSONDecodeError):
            continue                                   # bound as bytes; not a JSON tree to follow
        for text in strings(data):
            if not text.startswith(str(D) + '/') or '\n' in text:
                continue
            ref = Path(text)
            if ref.is_file():
                found.add(str(ref))
                if ref.suffix == '.json':
                    todo.append(ref)
    return found


def guard_pins():
    """The files the guard's own PINS name, bound as 667 bound them for 666."""
    rows = {}
    for node in ast.parse(GUARD.read_text(encoding='utf-8')).body:
        if isinstance(node, ast.Assign) and any(isinstance(t, ast.Name) and t.id == 'PINS' for t in node.targets):
            for rel, digest in ast.literal_eval(node.value).items():
                assert sha(D/rel) == digest, ('guard pin changed', rel)
                rows[str(D/rel)] = digest
    return rows


def leaves(value, at=()):
    if isinstance(value, dict) and value:
        out = {}
        for k, v in value.items():
            out.update(leaves(v, at + (str(k),)))
        return out
    if isinstance(value, list) and value:
        out = {}
        for i, v in enumerate(value):
            out.update(leaves(v, at + (str(i),)))
        return out
    return {at: (type(value).__name__, value)}


def changed(before, after):
    """The leaves (as a/b/c) whose type or value differs, or that only one side has."""
    a, b = leaves(before), leaves(after)
    return sorted('/'.join(k) for k in set(a) | set(b) if a.get(k, MISSING) != b.get(k, MISSING))


def text_of(value, newline):
    return json.dumps(value, indent=2) + ('\n' if newline else '')


def parse_exact(path, newline):
    """The JSON at path, refused unless writing it back the same way gives the same bytes."""
    raw = Path(path).read_text(encoding='utf-8')
    value = json.loads(raw)
    assert text_of(value, newline) == raw, ('does not round-trip, so a rebind would change more than its fields', path)
    return value


def rebind_map(data):
    out = copy.deepcopy(data)
    hold = out['visual_holds'][HOLD_INDEX]
    assert {k: hold[k] for k in HOLD} == HOLD and (hold['sha256'], hold['line']) == OLD_SOURCE, 'not the 06_1 record'
    hold['sha256'], hold['line'] = NEW_SOURCE
    assert changed(data, out) == CHANGES['map']
    return out


def rebind_units(data, map_sha):
    out = copy.deepcopy(data)
    contract = out['direction_contract']
    assert contract == dict(schema_version=1, topic='apportionment', script_sha256=SCRIPT_SHA,
                            map=dict(path=MAP, sha256=MAP_SHA)), 'the unit plan binds another contract'
    assert data['script_sha256'] == SCRIPT_SHA
    contract['map'] = dict(path=NEW_MAP, sha256=map_sha)
    assert changed(data, out) == CHANGES['units']
    return out


def rebind_report(data, units_sha, map_sha):
    out = copy.deepcopy(data)
    meta = out['meta']
    assert meta['units_sha256'] == UNITS_SHA and meta['script_sha256'] == SCRIPT_SHA, 'report binds another plan'
    assert meta['direction_contract']['binding']['contract']['map'] == dict(path=MAP, sha256=MAP_SHA), 'report map'
    meta['units_sha256'] = units_sha
    meta['direction_contract']['binding']['contract']['map'] = dict(path=NEW_MAP, sha256=map_sha)
    assert changed(data, out) == CHANGES['report']
    return out


def rebind_assembly(data, units_sha, report_sha):
    out = copy.deepcopy(data)
    assert out['manifest']['units_sha256'] == UNITS_SHA, 'assembly binds another plan'
    out['manifest']['units_sha256'] = units_sha
    out['report_sha256'] = report_sha
    assert changed(data, out) == CHANGES['assembly']
    return out


def marker_once_at(path, line, marker):
    lines = Path(path).read_text(encoding='utf-8').split('\n')
    return 0 < line <= len(lines) and lines[line - 1].strip() == marker and [x.strip() for x in lines].count(marker) == 1


def check_records(workspace, data, skip=()):
    """Every hold and helper record names its file's current hash and the one line holding its marker."""
    checked = []
    for kind in ('visual_holds', 'helpers'):
        for i, record in enumerate(data[kind]):
            if (kind, i) in skip:
                continue
            path = workspace/record['path']
            ok = sha(path) == record['sha256'] and marker_once_at(path, record['line'], record['marker'])
            assert ok, ('record does not match its source', kind, i, record['path'])
            checked.append('%s/%d' % (kind, i))
    return checked


def write_new(path, data, mode='x'):
    with Path(path).open(mode) as f:
        f.write(data)


def copy_new(src, dst):
    with Path(src).open('rb') as s, Path(dst).open('xb') as t:
        shutil.copyfileobj(s, t)
    assert sha(src) == sha(dst), dst


def evidence748(build):
    """Finished build748, read only. Returns (rows to bind, qualification748, binary, workspace)."""
    job = JOBS/build
    r = success(job, '748')
    assert r['command'] == [PYTHON, '-I', '-B', str(BUILD_HELPER), 'run', '--job', build], ('748 command', r['command'])
    assert sha(BUILD_HELPER) == BUILD_SHA, '748 helper is not the sealed one'
    manifest = Q/(build + '-inputs.json')
    assert sha(manifest) == r['inputs_sha256'], '748 receipt binds another manifest'
    assert manifest_rows(manifest).get(str(BUILD_HELPER)) == BUILD_SHA, '748 manifest binds another helper'
    q = read(job/'qualification748.json')
    binary, workspace = job/'apportionment-native748', job/'workspace'
    assert q['schema'] == 'apportionment-ch06-build748' and q['inputs_sha256'] == r['inputs_sha256'], '748 record'
    assert q['binary'] == str(binary) and sha(binary) == q['binary_sha256'], '748 binary differs from its record'
    assert q['workspace'] == str(workspace), '748 workspace'
    assert q['workspace_listing'] == list(WORKSPACE731) and q['crate_listing'] == list(CRATE731), '748 listings'
    assert q['tests_passed'] == 33 and q['clippy_passed'] is True and q['facts'] == 443, '748 checks'
    assert q['facts_equal_656'] is True and q['oracle_equal_656'] is True, '748 facts'
    assert q['chapters'] == CHAPTERS and len(q['cues']) == CUES and q['cues_said_in_script_order'] is True, '748 cues'
    crate = workspace/TOPIC
    for rel, digest in CANDIDATES722.items():
        assert sha(crate/rel) == digest, ('748 workspace is not the exact 722 source', rel)
    assert sha(crate/'script.tsv') == SCRIPT_SHA and script_ids(crate) == q['cues'], '748 script'
    rows = {str(p): sha(p) for p in (job/'resource-receipt.json', job/'qualification748.json', job/'preflight748.json',
                                     binary, manifest, BUILD_HELPER)}
    return rows, q, binary, workspace


def evidence656():
    """656's copies of the original unit plan, map and not_every_time.rs, and 667's tools beside them."""
    success(BUILD656, '656')
    source = read(BUILD656/'qualification.json')['source']
    rows = {}
    for rel, digest in ((UNITS, UNITS_SHA), (MAP, MAP_SHA), (HOLD['path'], OLD_SOURCE[0])):
        path = BUILD656/'workspace'/rel
        assert source.get('workspace/' + rel) == digest and sha(path) == digest, ('656 copy', rel)
        rows[str(path)] = digest
    assert marker_once_at(BUILD656/'workspace'/HOLD['path'], OLD_SOURCE[1], HOLD['marker']), '656 record line 58'
    assert read(RUNTIME667) == [dict(path=str(BUILD656/'workspace/tools'/n), sha256=h) for n, h in TOOLS.items()], '667'
    for name, digest in TOOLS.items():
        for path in (BUILD656/'workspace/tools'/name, ASSEMBLY664/'repo/tools'/name):
            assert sha(path) == digest, ('667 tool', path)
            rows[str(path)] = digest
    for path in (BUILD656/'resource-receipt.json', BUILD656/'qualification.json', RUNTIME667):
        rows[str(path)] = sha(path)
    return rows


def evidence666():
    """Finished 666 and the exact files it bound: its manifest names every file under 664's repo."""
    r = success(TIMELINE666, '666')
    assert sha(INPUTS666) == r['inputs_sha256'], '666 receipt binds another manifest'
    rows = {str(TIMELINE666/'resource-receipt.json'): sha(TIMELINE666/'resource-receipt.json'), str(INPUTS666): sha(INPUTS666)}
    return rows, manifest_rows(INPUTS666)


def evidence664(ids, bound666):
    """664's narration: success, 93 WAVs at their assembly hashes, phrase marks at 566's hash, all as 666 bound them.
    So is every file under 664's repo that the report and assembly name at any depth (receipts, source WAVs)."""
    success(ASSEMBLY664, '664')
    record = read(ASSEMBLY664/'relocation664.json')
    assert record['full_coherent_verification'] is True and record['cues'] == CUES, '664 record'
    assert record['wavs_byte_identical'] is True and record['phrase_timings_byte_identical'] is True, '664 record'
    names = sorted(p.name for p in N664.iterdir())
    assert names == sorted([c + '.wav' for c in ids] + ['assembly.json', 'assembly.json.before-path664',
                                                          'phrase-timings.json', 'report.json',
                                                          'report.json.before-path664']), ('664 narration', names)
    assembly = read(N664/'assembly.json')
    assert assembly['report_sha256'] == sha(N664/'report.json') and sorted(assembly['outputs']) == sorted(ids), '664'
    for cue in ids:
        assert sha(N664/(cue + '.wav')) == assembly['outputs'][cue]['sha256'], ('664 recording', cue)
    assert sha(N664/'phrase-timings.json') == PHRASE_SHA, '664 phrase marks'
    rows = {}
    for path in N664.iterdir():
        assert bound666.get(str(path)) == sha(path), ('not the narration file 666 used', path.name)
        rows[str(path)] = bound666[str(path)]
    for path in (ASSEMBLY664/'resource-receipt.json', ASSEMBLY664/'relocation664.json'):
        rows[str(path)] = sha(path)
    for path in closure([N664/'report.json', N664/'assembly.json']):
        rows[path] = sha(path)
        if Path(path).is_relative_to(ASSEMBLY664/'repo'):
            assert bound666.get(path) == rows[path], ('not the file 666 bound', path)
    return rows


def host_rows(*paths):
    rows = {str(HELPER): sha(HELPER), str(GUARD): sha(GUARD), str(STOPPER): sha(STOPPER)}
    for path in (PYTHON,) + paths:
        rows[str(Path(path).resolve())] = sha(Path(path).resolve())
    rows.update(guard_pins())
    return rows


def bound_prepare(build):
    """Every file the prepare phase reads. 748's narration_units.json is left out because this job moves it aside;
    656's identical copy is bound in its place, and run() checks 748's against UNITS_SHA."""
    rows, q748, binary, workspace = evidence748(build)
    assert listing(workspace) == WORKSPACE731, '748 workspace is not as built (an earlier install, or a change)'
    for rel in ('tools', NEW_MAP, UNITS_ASIDE):
        assert not (workspace/rel).exists(), ('already present in 748 workspace', rel)
    for rel, digest in tree_rows(workspace).items():
        if rel != UNITS:
            rows[str(workspace/rel)] = digest
    assert sha(workspace/UNITS) == UNITS_SHA and sha(workspace/MAP) == MAP_SHA, '748 plan or map'
    rows.update(evidence656())
    more, bound666 = evidence666()
    rows.update(more)
    rows.update(evidence664(q748['cues'], bound666))
    rows.update(host_rows())
    return rows, q748, binary, workspace


def check_rebind_record(prepare, workspace):
    """Finished prepare744 and its narration, as rebind744.json records them."""
    job = JOBS/prepare
    r = success(job, '744 prepare')
    record = read(job/'rebind744.json')
    narration = job/'narration'
    assert record['schema'] == 'apportionment-ch06-prepare744' and record['narration'] == str(narration), 'rebind744'
    assert record['workspace'] == str(workspace) and record['installed_listing'] == list(INSTALLED744), 'rebind744'
    assert record['map']['after']['sha256'] == NEW_MAP_SHA and record['units']['after']['sha256'] == NEW_UNITS_SHA
    assert record['cues_reconstructed'] == CUES and record['delivery']['verified_cues'] == CUES, 'rebind744'
    assert tree_rows(narration) == record['narration_files'], '744 narration changed since prepare'
    assert sha(narration/'phrase-timings.json') == PHRASE_SHA, '744 phrase marks'
    rows = {str(job/'resource-receipt.json'): sha(job/'resource-receipt.json'),
            str(job/'rebind744.json'): sha(job/'rebind744.json'), str(job/'install744.json'): sha(job/'install744.json')}
    rows[str(Q/(prepare + '-inputs.json'))] = r['inputs_sha256']
    assert sha(Q/(prepare + '-inputs.json')) == r['inputs_sha256'], '744 prepare manifest'
    for rel, digest in record['narration_files'].items():
        rows[str(narration/rel)] = digest
    return rows, narration


def evidence666_tables():
    """666's 45 tables at the hashes comparison666.json records, and its frame counts."""
    record = read(TIMELINE666/'comparison666.json')
    assert record['full_normalized_tables_equal'] is True and record['total_frames'] == FRAMES666, '666 comparison'
    assert list(record['chapters']) == CHAPTERS and record['score_qa_passed'] == 15, '666 comparison'
    rows = {str(TIMELINE666/'comparison666.json'): sha(TIMELINE666/'comparison666.json')}
    for ch in CHAPTERS:
        for ext in TABLES:
            path = TIMELINE666/'timeline'/(ch + '.' + ext)
            assert sha(path) == record['chapters'][ch]['tables'][ext], ('666 table changed', ch, ext)
            rows[str(path)] = record['chapters'][ch]['tables'][ext]
    assert sum(record['chapters'][ch]['frames'] for ch in CHAPTERS) == FRAMES666, '666 frames'
    return rows, record


def bound_timeline(build, prepare):
    rows, q748, binary, workspace = evidence748(build)
    assert listing(workspace) == INSTALLED744 and listing(workspace/TOPIC) == CRATE744, '748 workspace is not as installed'
    for rel, digest in tree_rows(workspace).items():
        rows[str(workspace/rel)] = digest
    more, narration = check_rebind_record(prepare, workspace)
    rows.update(more)
    rows.update(evidence656())
    more, bound666 = evidence666()
    rows.update(more)
    rows.update(evidence664(q748['cues'], bound666))
    for path in closure([narration/'report.json', narration/'assembly.json']):
        rows[path] = sha(path)
    tables, record666 = evidence666_tables()
    rows.update(tables)
    assert sorted(p.stem for p in STEMS_QA.glob('*.json')) == CHAPTERS, 'score QA files'
    for ch in CHAPTERS:
        assert read(STEMS_QA/(ch + '.json'))['passed'] is True, ('score QA', ch)
        rows[str(STEMS_QA/(ch + '.json'))] = sha(STEMS_QA/(ch + '.json'))
    rows.update(host_rows(FFMPEG, FFPROBE))
    return rows, q748, binary, workspace, narration, record666


def read_receipt(job):
    for _ in range(10):                                # the guard rewrites it every 10 s while we run
        try:
            return json.loads((job/'resource-receipt.json').read_text())
        except json.JSONDecodeError:
            time.sleep(0.5)
    raise SystemExit('744 refused: the guard receipt could not be read')


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


def own_group():
    """The helper leads the group the guard made with start_new_session, so the group is this job and nothing else."""
    pid = os.getpid()
    assert os.getpgid(0) == pid and os.getsid(0) == pid, 'run 744 as the guard command itself, not under a wrapper'


def end_group(reason):
    """End the whole owned group, this helper last. Never returns."""
    for sig in STOP_SIGNALS:
        signal.signal(sig, signal.SIG_IGN)            # the helper outlives its own SIGTERM to write the record
    group = os.getpgid(0)
    os.killpg(group, signal.SIGTERM)                   # every step's process: all but this helper
    child = STATE['child']
    if child is not None:
        try:
            child.wait(timeout=TERM_GRACE)
        except subprocess.TimeoutExpired:
            pass
    record = dict(reason=reason, step=STATE['step'], group=group, child_exit=None if child is None else child.poll(),
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


def run_step(job, crate, name, argv, seconds, extra):
    """Build 521's run_step. From launch until the step is waited, any exception ends the group."""
    start = time.monotonic()
    with (job/(name + '.stdout')).open('xb') as o, (job/(name + '.stderr')).open('xb') as e:
        STATE['step'] = name
        try:
            STATE['child'] = subprocess.Popen(list(map(str, argv)), cwd=crate, stdout=o, stderr=e,
                                              env=dict(os.environ, **(extra or {})))
            code = STATE['child'].wait(timeout=seconds)
        except BaseException as error:
            end_group(repr(error))
        STATE['child'] = None
    TIMES[name] = round(time.monotonic() - start, 1)
    assert code == 0, (name, code)
    return (job/(name + '.stdout')).read_bytes()


def check_host():
    assert sys.version_info >= (3, 11) and sys.flags.isolated and sys.dont_write_bytecode, 'run with python3 -I -B'
    assert platform.system() == 'Linux' and platform.machine() == 'x86_64', 'this helper runs on the CSD only'


def write_manifest(args, rows):
    manifest = Q/(args.job + '-inputs.json')
    files = [dict(path=path, sha256=rows[path]) for path in sorted(rows)]
    check_files(files)
    with manifest.open('x') as f:
        json.dump(dict(schema='apportionment-ch06-%s744-inputs' % args.phase, job=args.job, files=files), f, indent=1)
    print(json.dumps(dict(manifest=str(manifest), sha256=sha(manifest), files=len(files))))
    return 0


def prepare_inputs(args):
    assert not (JOBS/args.job).exists() and not (Q/(args.job + '-inputs.json')).exists(), 'no duplicate job'
    if args.phase == 'prepare':
        rows = bound_prepare(args.build)[0]
    else:
        rows = bound_timeline(args.build, args.prepare)[0]
    return write_manifest(args, rows)


def start(args, bound):
    """Common start of both phases under the guard. Returns the job, its manifest rows and what bound() returned."""
    own_group()
    for sig in STOP_SIGNALS:
        signal.signal(sig, on_stop)
    job = JOBS/args.job
    manifest = Q/(args.job + '-inputs.json')
    guard = guard_binding(job, manifest)
    STATE['job'] = job
    files = read(manifest)['files']
    check_files(files)
    result = bound()
    have = {row['path']: row['sha256'] for row in files}
    assert result[0] == have, ('the manifest does not bind exactly what this phase reads',
                               sorted(set(result[0]) ^ set(have))[:20])
    return job, files, guard, result


def run_prepare(args):
    job, files, guard, (_, q748, binary, workspace) = start(args, lambda: bound_prepare(args.build))
    crate = workspace/TOPIC
    ids = q748['cues']

    def step(name, argv, cwd):
        return run_step(job, cwd, name, argv, VERIFY_SECONDS, None)

    # Sources: the old record was right for 656's file; every other record is right for 748's files.
    old_map = parse_exact(workspace/MAP, True)
    assert sha(workspace/MAP) == MAP_SHA and sha(workspace/UNITS) == UNITS_SHA, '748 map or unit plan'
    others = check_records(workspace, old_map, skip={('visual_holds', HOLD_INDEX)})
    assert len(old_map['visual_holds']) == 15 and len(old_map['helpers']) == 3 and len(others) == 17, 'record count'
    assert marker_once_at(workspace/HOLD['path'], NEW_SOURCE[1], HOLD['marker']), 'marker not once at line 102'

    # Rebind, in the job folder only.
    tree = job/'rebind'
    (tree/'tools').mkdir(parents=True)
    (tree/TOPIC/'docs').mkdir(parents=True)
    for name in TOOLS:
        copy_new(BUILD656/'workspace/tools'/name, tree/'tools'/name)
    new_map = rebind_map(old_map)
    write_new(tree/NEW_MAP, text_of(new_map, True))
    assert sha(tree/NEW_MAP) == NEW_MAP_SHA, ('rebound map', sha(tree/NEW_MAP))
    after = check_records(workspace, new_map)            # all 18 records, the rebound one included, match 748's files
    assert len(after) == 18 and set(after) - set(others) == {'visual_holds/%d' % HOLD_INDEX}, after
    units = parse_exact(workspace/UNITS, True)
    new_units = rebind_units(units, NEW_MAP_SHA)
    write_new(tree/UNITS, text_of(new_units, True))
    assert sha(tree/UNITS) == NEW_UNITS_SHA, ('rebound unit plan', sha(tree/UNITS))

    narration = job/'narration'
    narration.mkdir()
    for name in [c + '.wav' for c in ids] + ['phrase-timings.json']:
        copy_new(N664/name, narration/name)
    report = parse_exact(N664/'report.json', False)
    assembly = parse_exact(N664/'assembly.json', False)
    assert assembly['report_sha256'] == sha(N664/'report.json')
    write_new(narration/'report.json', text_of(rebind_report(report, NEW_UNITS_SHA, NEW_MAP_SHA), False))
    write_new(narration/'assembly.json', text_of(rebind_assembly(assembly, NEW_UNITS_SHA, sha(narration/'report.json')),
                                                 False))
    copied = {p.name for p in N664.iterdir() if (narration/p.name).exists() and sha(p) == sha(narration/p.name)}
    assert copied == {c + '.wav' for c in ids} | {'phrase-timings.json'}, 'recordings or phrase marks differ from 664'
    script = crate/'script.tsv'
    step('verify-copy', [PYTHON, '-I', '-B', tree/'tools/coherent_narration.py', 'verify', narration, '--script',
                         script, '--units', tree/UNITS, '--require-directions'], tree)

    # Install beside 748's compiled topic. Nothing is deleted or overwritten.
    tools = workspace/'tools'
    tools.mkdir()
    for name in TOOLS:
        copy_new(BUILD656/'workspace/tools'/name, tools/name)
    copy_new(tree/NEW_MAP, workspace/NEW_MAP)
    assert not (workspace/UNITS_ASIDE).exists()
    os.rename(workspace/UNITS, workspace/UNITS_ASIDE)
    copy_new(tree/UNITS, workspace/UNITS)
    installed = {rel: sha(workspace/rel) for rel in ['tools/' + n for n in TOOLS] + [NEW_MAP, UNITS, UNITS_ASIDE]}
    with (job/'install744.json').open('x') as f:
        json.dump(dict(workspace=str(workspace), files=installed, moved_aside={UNITS: UNITS_ASIDE},
                       note='Installed by prepare744 before its recheck; kept whatever the recheck shows.'), f, indent=2)
    assert installed == dict({'tools/' + n: h for n, h in TOOLS.items()},
                             **{NEW_MAP: NEW_MAP_SHA, UNITS: NEW_UNITS_SHA, UNITS_ASIDE: UNITS_SHA}), installed

    # Recheck with the installed tools and plan, as the binary will call them.
    step('verify-installed', [PYTHON, '-I', '-B', tools/'coherent_narration.py', 'verify', narration, '--script', script,
                              '--units', crate/'narration_units.json', '--require-directions'], crate)
    keys = [x for ch in CHAPTERS for x in ('--chapter-key', ch)]
    delivery = json.loads(step('delivery', [PYTHON, '-I', '-B', tools/'narration_delivery.py', '--topic', crate,
                                            '--narration', narration, '--consumer', 'render', '--gain-db', '0', *keys],
                               crate))
    assert delivery['consumer'] == 'render' and delivery['topic'] == 'apportionment', delivery
    assert delivery['verified_cues'] == delivery['selected_cues'] == CUES and delivery['chapter_scope'] is None, delivery
    assert delivery['plan_mode'] == 'canonical' and delivery['promotion'] is False, delivery
    check_files(files)
    assert listing(workspace) == INSTALLED744 and listing(crate) == CRATE744, '748 workspace after install'
    with (job/'rebind744.json').open('x') as out:
        json.dump(dict(
            schema='apportionment-ch06-prepare744', node=platform.node(), guard=guard, build=args.build,
            binary=str(binary), binary_sha256=sha(binary), workspace=str(workspace), workspace_listing_before=WORKSPACE731,
            installed_listing=INSTALLED744, crate_listing=CRATE744, installed=installed, narration=str(narration),
            narration_files=tree_rows(narration), cues_reconstructed=CUES, wavs_byte_identical_to_664=CUES,
            phrase_timings_sha256=PHRASE_SHA,
            hold=dict(index=HOLD_INDEX, cue=HOLD['cue'], next_cue=HOLD['next_cue'], path=HOLD['path'],
                      marker=HOLD['marker'], before=dict(sha256=OLD_SOURCE[0], line=OLD_SOURCE[1]),
                      after=dict(sha256=NEW_SOURCE[0], line=NEW_SOURCE[1])),
            records_unchanged_and_checked=others, records_checked_after_rebind=after,
            map=dict(before=dict(path=MAP, sha256=MAP_SHA), after=dict(path=NEW_MAP, sha256=NEW_MAP_SHA),
                     changed=CHANGES['map']),
            units=dict(before=dict(path=UNITS_ASIDE, sha256=UNITS_SHA), after=dict(path=UNITS, sha256=NEW_UNITS_SHA),
                       changed=CHANGES['units']),
            report=dict(before=sha(N664/'report.json'), after=sha(narration/'report.json'), changed=CHANGES['report']),
            assembly=dict(before=sha(N664/'assembly.json'), after=sha(narration/'assembly.json'),
                          changed=CHANGES['assembly']),
            spans_sha256_unchanged=report['meta']['spans_sha256'],
            verify=dict(copy=sha(job/'verify-copy.stdout'), installed=sha(job/'verify-installed.stdout')),
            delivery=delivery, inputs_sha256=sha(Q/(args.job + '-inputs.json')), seconds=TIMES,
            scope='Metadata rebind of one hold record and its contract hashes; 667 tools installed beside the 748 topic; '
                  'the 93 retained recordings rebuilt sample for sample from their sources twice, and the delivery '
                  'check the binary runs. No render, timeline, picture, motion, audio, fine-sync, AV or release '
                  'acceptance, and no reuse of any retained render.'), out, indent=2)
    print(json.dumps(dict(passed=True, job=job.name, report=sha(narration/'report.json'))))
    return 0


def frames_of(path):
    head = dict(x.split('\t', 1) for x in path.read_text(encoding='utf-8').splitlines()
                if x.startswith(('chapter\t', 'frames\t', 'fps\t')))
    return head


def run_timeline(args):
    job, files, guard, (_, q748, binary, workspace, narration, record666) = start(
        args, lambda: bound_timeline(args.build, args.prepare))
    crate = workspace/TOPIC
    os.sched_setaffinity(0, {0, 1, 2, 3})
    os.environ['RAYON_NUM_THREADS'] = '4'
    extra = dict(PATH='/usr/bin:/bin')                 # python3 and ffmpeg as bound; the binary calls both by name
    chapters = run_step(job, crate, 'chapters', [binary, 'chapters'], STEP_SECONDS, extra).decode().splitlines()
    assert chapters == CHAPTERS == q748['chapters'], chapters
    out = job/'timeline'
    assert not out.exists()
    deadline = time.monotonic() + TIMELINE_SECONDS
    records, problems = {}, []
    for ch in CHAPTERS:
        print('TIMELINE744', ch, flush=True)
        run_step(job, crate, 'render-' + ch, [binary, 'render', ch, '--res', '64x36', '--fps', '60', '--lint',
                                              '--narration', narration, '--out', out, '--crf', '16', '--preset', 'slow',
                                              '--threads', '4'], max(1, deadline - time.monotonic()), extra)
        old = record666['chapters'][ch]
        equal, tables = {}, {}
        for ext in TABLES:
            ours = (out/(ch + '.' + ext)).read_text(encoding='utf-8')
            ref = (TIMELINE666/'timeline'/(ch + '.' + ext)).read_text(encoding='utf-8')
            equal[ext] = ours.replace(str(narration), str(N664)) == ref
            tables[ext] = sha(out/(ch + '.' + ext))
            if not equal[ext]:
                problems.append('%s %s differs from 666' % (ch, ext))
        head = frames_of(out/(ch + '.timeline.tsv'))
        assert head['chapter'] == ch and head['fps'] == '60', head
        frames = int(head['frames'])
        if frames != old['frames']:
            problems.append('%s has %d frames, 666 had %d' % (ch, frames, old['frames']))
        layout = sha(out/(ch + '.layout.tsv'))
        records[ch] = dict(frames=frames, frames_666=old['frames'], tables=tables, normalized_equal_666=equal,
                           layout_sha256=layout, layout_equal_666=layout == old['layout_sha256'],
                           layout_bytes=(out/(ch + '.layout.tsv')).stat().st_size)
        print('SCHEDULE744', ch, frames, 'equal' if all(equal.values()) and frames == old['frames'] else 'DIFFERS',
              flush=True)
    check_files(files)
    if listing(workspace) != INSTALLED744:
        problems.append('748 workspace changed during the renders')
    total = sum(r['frames'] for r in records.values())
    equal_tables = sum(v for r in records.values() for v in r['normalized_equal_666'].values())
    retained = [c for c in CHAPTERS if c != NEW_CHAPTER]
    result = dict(
        schema='apportionment-ch06-timeline744', node=platform.node(), guard=guard, build=args.build,
        prepare=args.prepare, binary=str(binary), binary_sha256=sha(binary), narration=str(narration),
        rebind744_sha256=sha(JOBS/args.prepare/'rebind744.json'), reference='timeline666 (apportionment-native656, '
        'narration 664)', chapters=records, total_frames=total, total_frames_666=FRAMES666, tables_compared=45,
        tables_equal=equal_tables, full_normalized_tables_equal=not problems, problems=problems,
        layout_differs=[c for c in CHAPTERS if not records[c]['layout_equal_666']], seconds=TIMES,
        inputs_sha256=sha(Q/(args.job + '-inputs.json')),
        provenance=dict(new=dict(chapter=NEW_CHAPTER, binary='apportionment-native748',
                                 note='A new 06 final must come from this binary with this narration. The old 06 '
                                      'final from apportionment-native656 stays preserved and is not this chapter.'),
                        retained=dict(chapters=retained, note='01 and 02 are local final632 renders (export 682); '
                                      '03-05 and 07-15 are CSD final672 renders from apportionment-native656. Equal '
                                      'schedules are a precondition for root to decide on reusing them, not that '
                                      'decision.')),
        scope='Native schedules (64x36 timeline renders) from apportionment-native748 against timeline666 after the '
              'narration path substitution. No final render, frame, picture, motion, audio, fine-sync, AV or '
              'release acceptance, and no reuse decision.')
    name = 'comparison744.json' if not problems else 'mismatch744.json'
    with (job/name).open('x') as f:
        json.dump(result, f, indent=2)
    print(json.dumps(dict(passed=not problems, job=job.name, record=name, problems=problems[:20])))
    return 0 if not problems else 1


def main():
    p = argparse.ArgumentParser(description=__doc__.split('\n', 1)[0])
    p.add_argument('mode', choices=('inputs', 'run'))
    p.add_argument('phase', choices=('prepare', 'timeline'))
    p.add_argument('--job', required=True)
    p.add_argument('--build', required=True)
    p.add_argument('--prepare')
    args = p.parse_args()
    assert re.fullmatch(JOB_NAMES[args.phase], args.job), 'job must be the %s744 name or that name plus -aN' % args.phase
    assert re.fullmatch(JOB_NAMES['build'], args.build), 'build must be csd-apportionment-ch06-build748 or plus -aN'
    if args.phase == 'timeline':
        assert args.prepare and re.fullmatch(JOB_NAMES['prepare'], args.prepare), 'timeline needs --prepare'
    else:
        assert args.prepare is None, 'prepare takes no --prepare'
    check_host()
    if args.mode == 'inputs':
        return prepare_inputs(args)
    return run_prepare(args) if args.phase == 'prepare' else run_timeline(args)


if __name__ == '__main__':
    raise SystemExit(main())
