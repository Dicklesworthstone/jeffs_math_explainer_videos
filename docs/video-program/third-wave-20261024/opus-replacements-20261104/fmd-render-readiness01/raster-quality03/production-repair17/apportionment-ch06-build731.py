"""Build 731 (CC0): guarded CSD build of apportionment with the four repair722 chapter 06 candidates. Build checks only.

Prepared only: it has not run on any host. It succeeds build656, which stays as it is. This helper reads 656's job
folder and writes nothing there. Root accepted repair722 for guarded qualification after review 725. That review's three
Lows stay open: F1, the narration check is not limited to 06_4; F2, the 06_3 tag starts move inside the fixed 1.5 s
group; F3, Cedar's wedge touches the top tick. No 722 Rust has been compiled, so a compile, test or Clippy stop is a
plausible first result. A stop is a finding for the source owner. A successor gets new names, and this file stays as
it is.

The source, CSD only:
  workspace  The 95 workspace files that finished build656 recorded in its qualification.json, copied into the job
             folder at those hashes. That is the 652/655 closure with 656's two edits (root members, and the 19 engine
             source lines in Cargo.lock). The four 722 candidates take the place of the four canonical files. Each
             canonical file must be at its 722 "before" hash and each candidate at its 722 hash. script.tsv and
             docs/description_numbers.tsv must be at their frozen hashes; facts.rs now reads the second at compile time.
  engine     Used in place, read only, from build656 through 656's own pristine-engine.toml, as build716 did.
  Nothing else changes. The pins were computed from exports 652/655 and the candidates. `inputs` refuses before
  writing anything if 656's recorded map, logs or outputs differ from them.

Steps, in order. Any failure stops the job, and there is no retry:
  tests      cargo test --bin apportionment: one result line, 33 passed, 0 failed, 0 ignored, 0 filtered out. The
             passed names are exactly the 30 that 656 passed (read from its tests.stdout) plus the three 722 tests.
  clippy     --all-targets --no-deps -D warnings. 656 linted the bin target only; this also lints test code.
  build      The binary, copied to apportionment-native731 in the job folder.
  facts      Binary facts equal oracle.py by key at 443 keys. Each output is byte-equal to 656's, and the oracle's is
             at the pinned hash. oracle.py --check exits 0 and reports that the census-year row and narration agree.
  chapters   The 15 keys, in the order 656 recorded.
  script     Binary script equals script.tsv: 93 unique cues, 4166 words, each owned by one chapter, numbered in order.
  static     Each cue said once, as say(&kit, stage, "NN_k"), outside test code, in a module chapters/mod.rs compiles.
             06 says 06_1 to 06_6, in order.
  cue probe  render all --scratch-voice at 64x36, 1 fps. Each chapter's ids as really said, joined, are the script order.
  glyphs     Every chapter builds and registers strings, with 0 failures. 06 registers the five strings that 722
             composes at run time, including the 1880 census line read from the embedded TSV row.

Process handling is build 521's and 716's. Every step stays in the helper's own group, which the guard created. On a
timeout, an exception inside a step or a stop signal sent to the helper, end_group() ends that group. That does not
include ffmpeg: the pinned engine starts it in a process group of its own, in the same session. Only the cue probe
reaches ffmpeg; the CSD guard's own stop covers it.

Usage, under guard_csd02 (see handoff732):  apportionment-ch06-build731.py inputs|run --job JOB
`inputs` checks everything it can without Cargo and writes the guard's manifest. `run` is the guard's command.
Outputs, all new files in the job folder: workspace/, preflight731.json, <step>.stdout/.stderr, any <step>.cleanup.json,
apportionment-native731, apportionment-previous (only if Cargo's target already held an apportionment binary),
cue-probe/, glyph-probe/, qualification731.json. Cargo writes only to D/work/target.

It authorizes no render beyond its own 64x36 cue probe, which uses estimated timing and no recordings. It generates no
narration and checks none (current narration: ElevenLabs Frederick Surrey). It does not rebind the 06_1 prediction hold.
It does not compare PCM or real-narration timelines, and does not establish that any retained render can be reused.
It accepts no picture, motion, audio, fine sync, AV or release result. Zero glyph failures can hide fallback faces, so
the pages still need reading.
"""
import argparse
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
import tomllib

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
JOBS = D/'jobs'
WORK = D/'work'
TARGET = WORK/'target'
TMP = WORK/'tmp'
CACHE = WORK/'cache'
BUILD656 = JOBS/'csd-apportionment-build656'
CANDIDATE_DIR = Q/'apportionment-ch06-repair722'
PYTHON = '/usr/bin/python3'
FFMPEG = '/usr/bin/ffmpeg'
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4, work_cap_gib=56, target_growth_cap_gib=20,
              sample_seconds=10)
JOB_NAME = r'csd-apportionment-ch06-build731(-a[0-9]+)?'
BIN = 'apportionment'
CRATE_KEY = 'workspace/explainer_corpus/apportionment/'

# Canonical path in the crate: (file in apportionment-ch06-repair722, 722 "before" sha256, candidate sha256).
CANDIDATES = {
    'src/chapters/not_every_time.rs': ('not_every_time.rs',
                                       'd3e88f7bcad146b9078ddc7d84588f6569339cd59cd3b2c225aa963bfadf7c94',
                                       '04499441ccde5be5f827b9768da26bbd0672f21e283728b298809f827d7225e9'),
    'src/facts.rs': ('facts.rs',
                     'b26bf1e409958a2ac9fb430f215cc19b4dbe6f3b350e0f28820a9056f1a18c8c',
                     'e9b86e26f7e285ddc397a8948462492c824e77a637a4904e501466a7ece78f1d'),
    'oracle.py': ('oracle.py',
                  'd9a5a6ab098cae541ad7b3ebd5aca90c0dfb43f34fe7f03ed7e91e94b2a9b803',
                  '8c1dd3af893059601f1c6a195a3292e7274d796edd4701adfefd6fb3d780c94f'),
    'docs/storyboard.md': ('storyboard.md',
                           '564e86db6c12c75a651bfcb6912619e2b3219337442095b64b78f2b49e930176',
                           '17a46b19463c07dd4713527574bcb46d56e8dc4dba6b648cc1daac4817f8ffbd'),
}
# Crate files 722 needs unchanged. The first two are frozen by handoff722; the rest are what Cargo and the tests read.
FROZEN = {'script.tsv': 'b9344e1123170e7e3898d8aea2123e0b659bcd571085008fc97894e7d69da624',
          'docs/description_numbers.tsv': 'f8e46262ea565e4f4c27554fde9b46fc88bcafefc5662a377b1d06cab31f9cc1',
          'Cargo.toml': 'c74357f03173101a19b83273ea8a82ab0d17c7d694c6486b157d251d5b1408f2',
          'Cargo.lock': 'a418725541675552020caaf6c8a20bfa59f5e0322c014aba760799cd50b6c8d8',   # after 656's edit
          'src/main.rs': 'ac6b1924cf525afd37bfc6680c78af33a055039a5e65388a83723ba1b939f95a',
          'src/chapters/mod.rs': '050e940041bc0f3e60faa9ec65815a7da93abda4228d814cfec0a09f908b10cf',
          'src/chapters/draw.rs': 'dd385ed5f45f09ffb8c0d8563829601bb147460bc16c7d4cbbf08245a7b23ba4'}
# (files, sha256 of sorted "<sha256>  <path>\n"): 656's whole recorded map, then this job's workspace and crate.
SOURCE656 = (4152, '77fd892aa2051eced56c47f1a7446288cecd584d80b88efd1133d48ae378e48e')
WORKSPACE731 = (95, 'eb860846c3347b0eaafeac3ce72437c3b0abc7704a15596a8944350b4e81d1d4')
CRATE731 = (49, '6a63d40f80e564fc81cae39c1adb72329b7312a3075ee9b1d4cbc25b236e5b1d')
PRISTINE_ENGINE = 'a96a57778f146726f641f4d0584d7c7887ec28196aedc979527bba918b754b13'   # 656's pristine-engine.toml
CARGO_CONFIG = '79fb8766d1f41365d64066e5b2e68b0fb89c50015425d109b155ccaf8eff2716'      # workspace .cargo/config.toml
ORACLE_SHA = '2d622f718124cc693cc298cda54784153d1e9705d377462925fce5440567b0fd'        # canonical = candidate, 443 lines
FACTS = 443
CUES = 93
WORDS = 4166
CENSUS_LINE = 'ok census year 1880: the sourced row and the narration agree'
CHAPTERS = ['01_one_more_seat', '02_shares', '03_largest_tails', '04_sixteenth', '05_dials', '06_not_every_time',
            '07_yardstick', '08_signposts', '09_cost', '10_rounding_rules', '11_webster_quota', '12_both_promises',
            '13_population', '14_no_rule', '15_recap']
NEW_CHAPTER = '06_not_every_time'
# What 722's 06 composes at run time, as kit.text receives it. The last line's year comes from the embedded TSV row.
NEW_STRINGS = ('16 seats → 17: does Ash get its seat back?', '+4/39 turn', '+9/39 turn', '+26/39 turn',
               '1880 census, C. W. Seaton: Hamilton apportionments for every House size from 275 to 350')
# The 30 tests 656 passed, derived from the canonical sources; `inputs` and `run` check them against 656's tests.stdout.
ORIGINAL_TESTS = tuple(sorted("""
chapters::both_promises::tests::the_count_panel_clears_the_axis_and_the_frame
chapters::both_promises::tests::the_count_row_keeps_its_parts_apart
chapters::both_promises::tests::the_labels_in_the_stretch_clear_the_markers_and_rows
chapters::both_promises::tests::the_think_ring_covers_no_card
chapters::draw::tests::a_full_row_fits_left_of_the_readouts
chapters::draw::tests::bands_split_whole_and_tail
chapters::draw::tests::nineteen_cards_fit_the_frame
chapters::draw::tests::sentences_split_like_the_narrator
chapters::shares::tests::the_drawn_division_is_the_share
chapters::tests::every_line_is_said_once_in_order
chapters::tests::script_and_registry_follow_the_storyboard
chapters::tests::spoken_words_follow_the_voice_rules
facts::tests::the_spoken_numbers
facts::tests::the_supposition_on_screen
model::tests::alabama_scan
model::tests::divisor_rules_at_the_film_houses
model::tests::every_tile_is_ruled_out_or_inside_exactly_once
model::tests::formatting
model::tests::hamilton_the_paradox_and_its_recovery
model::tests::hill_thresholds_sit_below_the_halfway_marks
model::tests::naive_rounding_misses_the_total
model::tests::no_state_of_the_picture_is_an_impossible_run
model::tests::signposts_and_windows
model::tests::tails_add_up_to_the_leftover_seats
model::tests::the_counts_on_screen_hold
model::tests::the_population_paradox
model::tests::the_quota_rule_skips_cedar_at_fifteen
model::tests::the_real_run_at_now_keeps_both_halves_of_the_promise
model::tests::the_supposition_starts_from_a_real_moment
model::tests::tiles_follow_jeffersons_list
""".split()))
NEW_TESTS = ('chapters::not_every_time::tests::the_printed_steps_are_the_exact_shares',
             'chapters::not_every_time::tests::the_question_reads_the_house_constants',
             'facts::tests::the_census_year_is_the_sourced_row')
RESULT = r'^test result: ok\. (\d+) passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;'

TOOLCHAIN = 'nightly-2026-08-31'
TRIPLE = 'x86_64-unknown-linux-gnu'
CARGO_VERSION = 'cargo 1.100.0-nightly (e8cb624d5 2026-08-22)'
RUSTC_LINES = ('release: 1.100.0-nightly', 'commit-hash: 90850177249efe0321573c569aec5d12b257f8d6',
               'commit-date: 2026-08-30', 'host: ' + TRIPLE)
CSD_RCH_WRAPPER = 'b854c9424332416075760b8b5fc8a43254e8eea8f38f14a57deea6126810f1b9'   # build656's two toolchain pins
CSD_RUSTC = '49f9fa06f3e2b93aba85273433fef67b984ddb4cacb4500de56e85f70ec2e339'
FORBIDDEN_ENV = ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER',
                 'RUSTDOC', 'RUSTDOCFLAGS', 'CARGO', 'CARGO_BUILD_TARGET')
FORBIDDEN_ENV_PREFIXES = ('CARGO_BUILD_RUSTC', 'CARGO_BUILD_RUSTFLAGS', 'CARGO_PROFILE_', 'CARGO_UNSTABLE_',
                          'CARGO_TARGET_', 'CARGO_ENV_', 'CARGO_PATCH', 'CARGO_SOURCE')
FORBIDDEN_TABLES = ('target', 'patch', 'source', 'env', 'unstable', 'replace')
FORBIDDEN_BUILD_KEYS = ('rustflags', 'rustdocflags', 'rustc', 'rustc-wrapper', 'rustc-workspace-wrapper',
                        'rustdoc', 'target', 'incremental')
ELF = b'\x7fELF'
STEP_SECONDS = 7200
COLD_SECONDS = 14400                                   # the first Cargo step compiles kit, score and the crate
TERM_GRACE = 6                                         # the guard waits 20 s after its SIGTERM, then SIGKILL
STOP_SIGNALS = (signal.SIGTERM, signal.SIGINT, signal.SIGHUP)
HELPER = Path(__file__).resolve()
STATE = dict(job=None, step=None, child=None)          # for end_group()'s record only; stopping never needs it
TIMES = {}


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def map_listing(rows):
    """The listing hash of a {relative path: sha256} map, as listing() computes it for a folder."""
    text = ''.join('%s  %s\n' % (rows[rel], rel) for rel in sorted(rows))
    return len(rows), hashlib.sha256(text.encode()).hexdigest()


def listing(top):
    """sha256 of sorted "<sha256>  <path>\\n" over every file; no links, no specials. CC1's and CC0's 492/506 rule."""
    rows = {}
    for parent, dirs, files in os.walk(top, followlinks=False):
        for name in dirs + files:
            path = Path(parent, name)
            if path.is_symlink():
                raise SystemExit('731 refused: symlink in the tree: %s' % path)
        for name in files:
            path = Path(parent, name)
            if not path.is_file():
                raise SystemExit('731 refused: not a regular file: %s' % path)
            rows[path.relative_to(top).as_posix()] = sha(path)
    return map_listing(rows)


def check_files(rows):
    for row in rows:
        assert sha(row['path']) == row['sha256'], ('input changed', row['path'])


def passed_tests(text):
    """The names libtest reported as passing, one per line; a repeated name is refused."""
    names = re.findall(r'^test (\S+)(?: - should panic)? \.\.\. ok$', text, re.M)
    assert len(set(names)) == len(names), 'a test name repeats'
    return names


def fact_rows(text):
    pairs = [line.split('\t', 1) for line in text.splitlines() if line.strip()]
    result = dict(pairs)
    assert len(result) == len(pairs), 'duplicate fact keys'
    return result


def evidence656():
    """Finished build656, read only: receipt, recorded map, patch file, binary and the logs 731 compares against.

    Returns ({path: sha256} to bind, 656's source map, 656's qualification). Nothing is written."""
    receipt, qualification = BUILD656/'resource-receipt.json', BUILD656/'qualification.json'
    r = read(receipt)
    assert r.get('admitted') is True and r.get('finished') and r.get('exit') == 0, ('656 receipt', r.get('exit'))
    assert r.get('inputs_sha256') == r.get('final_inputs_sha256'), '656 inputs changed'
    assert not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), '656 receipt'
    q = read(qualification)
    source = q['source']
    assert map_listing(source) == SOURCE656, ('656 recorded another source map', map_listing(source))
    assert q['chapters'] == CHAPTERS and q['tests'] == 30 and q['facts'] == FACTS, '656 qualification differs'
    for rel, (_, before, _) in CANDIDATES.items():
        assert source.get(CRATE_KEY + rel) == before, ('656 built another', rel)
    for rel, digest in FROZEN.items():
        assert source.get(CRATE_KEY + rel) == digest, ('656 built another', rel)
    assert source.get('workspace/.cargo/config.toml') == CARGO_CONFIG, '656 workspace config'
    rows = {}
    for rel, digest in source.items():
        path = BUILD656/rel
        assert not path.is_symlink() and sha(path) == digest, ('656 source changed since its build', rel)
        rows[str(path)] = digest
    config = BUILD656/'pristine-engine.toml'
    assert sha(config) == PRISTINE_ENGINE, '656 patch file differs'
    binary = BUILD656/'apportionment-native656'
    assert sha(binary) == q['binary_sha256'], '656 binary differs from its record'
    tests = (BUILD656/'tests.stdout').read_text()
    assert re.findall(RESULT, tests, re.M) == ['30'], '656 test result line'
    names = passed_tests(tests)
    assert sorted(names) == list(ORIGINAL_TESTS), ('656 passed another list', sorted(set(names) ^ set(ORIGINAL_TESTS)))
    oracle = (BUILD656/'oracle.stdout').read_bytes()
    assert hashlib.sha256(oracle).hexdigest() == ORACLE_SHA, '656 oracle output is not the pinned output'
    facts = fact_rows((BUILD656/'facts.stdout').read_text())
    assert len(facts) == FACTS and facts == fact_rows(oracle.decode()), '656 facts'
    for path in (receipt, qualification, config, binary, BUILD656/'tests.stdout', BUILD656/'facts.stdout',
                 BUILD656/'oracle.stdout'):
        rows[str(path)] = sha(path)
    return rows, source, q


def candidates():
    rows = {}
    for name, _, after in CANDIDATES.values():
        path = CANDIDATE_DIR/name
        assert path.is_file() and not path.is_symlink() and sha(path) == after, ('722 candidate missing or changed', name)
        rows[str(path)] = after
    return rows


def prepare(job, source):
    """The job's own workspace: 656's recorded workspace files at their hashes, the four candidates in place of the four
    canonical files. Each file is copied, never linked; nothing in 656 or in the candidate folder is written."""
    swap = {CRATE_KEY + rel: CANDIDATE_DIR/name for rel, (name, _, _) in CANDIDATES.items()}
    for key in sorted(source):
        if not key.startswith('workspace/'):
            continue
        out = job/key
        out.parent.mkdir(parents=True, exist_ok=True)
        with swap.get(key, BUILD656/key).open('rb') as src, out.open('xb') as dst:
            shutil.copyfileobj(src, dst)
    workspace, crate = job/'workspace', job/CRATE_KEY
    assert listing(workspace) == WORKSPACE731, 'the job workspace is not 656 with the four candidates'
    assert listing(crate) == CRATE731, 'the job crate is not 656 with the four candidates'
    for rel, (_, _, after) in CANDIDATES.items():
        assert sha(crate/rel) == after, rel
    return workspace, crate


def script_rows(crate):
    rows = []
    for raw in (crate/'script.tsv').read_text().split('\n'):
        line = raw.rstrip('\r')
        if line.strip() and not line.startswith('#'):
            key, text = line.split('\t')
            rows.append((key.strip(), text.strip()))
    return rows


def check_script(rows, chapters):
    """Unique cues, the pinned count and words, one owning chapter per cue, numbered 1..k within each chapter, in order."""
    ids = [key for key, _ in rows]
    assert len(ids) == CUES and len(set(ids)) == len(ids), len(ids)
    assert sum(len(text.split()) for _, text in rows) == WORDS, 'word count'
    assert all(re.fullmatch(r'[0-9]{2}_[1-9][0-9]*', key) for key in ids), 'cue id form'
    owned = {key: [] for key in chapters}
    for key in ids:
        owners = [c for c in chapters if c.startswith(key.split('_')[0] + '_')]
        assert len(owners) == 1, (key, owners)
        owned[owners[0]].append(key)
    assert all(owned.values()), 'a registered chapter has no cue'
    assert ids == [key for c in chapters for key in owned[c]], 'cues are not in chapter order'
    for keys in owned.values():
        assert [int(k.split('_')[1]) for k in keys] == list(range(1, len(keys) + 1)), keys
    return owned


def static_cues(crate, ids):
    """Each cue said exactly once, as say(&kit, stage, "NN_k"), outside test code, in a module chapters/mod.rs compiles.
    This film's say() takes the kit and the stage first; draw.rs defines it and says no cue itself."""
    chapters = crate/'src/chapters'
    modules = re.findall(r'^(?:pub(?:\(crate\))? )?mod (\w+);', (chapters/'mod.rs').read_text(), re.M)
    said = {}
    for module in modules:
        text = (chapters/(module + '.rs')).read_text().split('\n#[cfg(test)]\nmod tests {')[0]
        assert not re.search(r'\bsay\(&kit, stage, [^"]', text), ('a cue that is not a literal', module)
        said[module] = re.findall(r'\bsay\(&kit, stage, "([0-9]{2}_[0-9]+)"\)', text)
    flat = [cue for module in modules for cue in said[module]]
    assert sorted(flat) == sorted(ids), ('static cue coverage', sorted(set(ids) ^ set(flat)), len(flat))
    assert said.get('not_every_time') == ['06_%d' % k for k in range(1, 7)], ('06 says', said.get('not_every_time'))
    return modules


def said_cues(out, chapters, owned, ids):
    """The cue probe's rows: each chapter says only its own cues, at rising times; joined they are the script."""
    seen = []
    for key in chapters:
        rows = [line.split('\t') for line in (out/(key + '.cues.tsv')).read_text().split('\n') if line.strip()]
        assert all(len(row) == 3 for row in rows), key
        names = [row[0] for row in rows]
        assert names == owned[key], (key, names, owned[key])
        starts = [float(row[1]) for row in rows]
        assert all(a < b for a, b in zip(starts, starts[1:])), (key, starts)
        assert all(float(row[2]) > 0 for row in rows), key
        seen += names
    assert seen == ids, 'said order differs from the script'
    return seen


def check_glyphs(stdout, out, chapters):
    lines = stdout.splitlines()
    registered = [m for m in (re.fullmatch(r'glyph registry (\S+): (\d+) sources -> .*', l) for l in lines) if m]
    assert [m[1] for m in registered] == chapters, [m[1] for m in registered]
    assert all(int(m[2]) > 0 for m in registered), 'a chapter registered no strings'
    tail = [m for m in (re.fullmatch(r'glyph probe: (\d+) sources, 0 failures -> .*', l) for l in lines) if m]
    assert len(tail) == 1, lines[-3:]
    assert (out/'failures.txt').read_bytes() == b'', 'failures.txt is not empty'
    rows = [line.split('\t', 2) for line in (out/'registry.tsv').read_text().split('\n') if line]
    assert len(rows) == int(tail[0][1]) and all(len(row) == 3 and row[0] in ('Text', 'Tex', 'Mono') for row in rows), 'registry'
    allowed = set(chapters) | {'supplement'}
    assert all(set(row[1].split(',')) <= allowed for row in rows), 'registry names an unknown chapter'
    in_new = {row[2] for row in rows if row[0] == 'Text' and NEW_CHAPTER in row[1].split(',')}
    missing = [s for s in NEW_STRINGS if json.dumps(s, ensure_ascii=False) not in in_new]
    assert not missing, ('06 did not register', missing)
    pages = {p.relative_to(out).as_posix(): sha(p) for p in sorted(Path(a, n) for a, _, ns in os.walk(out/'pages') for n in ns)}
    assert pages, 'no probe pages'
    return dict(sources=int(tail[0][1]), per_chapter={m[1]: int(m[2]) for m in registered}, failures=0,
                registry_sha256=sha(out/'registry.tsv'), pages=pages, new_strings_in_06=list(NEW_STRINGS))


def cargo_home():
    home = Path(os.environ.get('CARGO_HOME') or Path.home()/'.cargo')
    assert home.is_absolute(), home
    return home


def config_problems(home, crate, bound):
    """Only the job workspace's config, at its pinned hash, and a harmless CARGO_HOME config may reach Cargo."""
    out, seen = [], {}
    for name in ('config', 'config.toml'):
        path = home/name
        if path.exists():
            data = tomllib.loads(path.read_text())
            seen[str(path)] = dict(sha256=sha(path), profile=data.get('profile', {}))
            out += ['%s sets [%s]' % (path, t) for t in FORBIDDEN_TABLES if t in data]
            out += ['%s sets build.%s' % (path, k) for k in FORBIDDEN_BUILD_KEYS if k in data.get('build', {})]
            out += ['%s sets [profile.%s]' % (path, p) for p in data.get('profile', {}) if p not in ('dev', 'test')]
    allowed = {str(p) for p in bound}
    for parent in crate.parents:
        for name in ('config', 'config.toml'):
            path = parent/'.cargo'/name
            if path.exists() and path.parent != home:
                if str(path) not in allowed:
                    out.append('an unbound Cargo config is on the crate path: %s' % path)
                elif sha(path) != CARGO_CONFIG:
                    out.append('the workspace Cargo config changed: %s' % path)
                else:
                    seen[str(path)] = dict(sha256=sha(path))
    return out, seen


def env_problems():
    bad = [k for k in os.environ if k in FORBIDDEN_ENV or (k.startswith(FORBIDDEN_ENV_PREFIXES) and k != 'CARGO_TARGET_DIR')]
    return ['forbidden environment: %s' % ', '.join(sorted(bad))] if bad else []


def text(argv, env):
    return subprocess.run(list(map(str, argv)), capture_output=True, text=True, timeout=120, env=env,
                          check=True).stdout.strip()


def native_binary(path):
    with Path(path).open('rb') as f:
        return f.read(len(ELF)) == ELF


def toolchain():
    """Never a shim: cargo-rch-real when RCH has replaced bin/cargo. 656's nightly, by version, commit and 656's pins."""
    rustup = Path.home()/'.cargo/bin/rustup'
    rustc = Path(subprocess.check_output([str(rustup), 'which', '--toolchain', TOOLCHAIN, 'rustc'], text=True,
                                         timeout=120).strip())
    bin_dir = rustc.parent
    assert bin_dir.parent.name == TOOLCHAIN + '-' + TRIPLE, ('wrong toolchain', bin_dir)
    real = bin_dir/'cargo-rch-real'
    cargo = real if real.is_file() else bin_dir/'cargo'
    tools = dict(cargo=cargo, rustc=rustc, cargo_clippy=bin_dir/'cargo-clippy', clippy_driver=bin_dir/'clippy-driver')
    for name, path in tools.items():
        assert path.is_file() and native_binary(path), ('%s is not a native binary (a shim?)' % name, path)
    assert sha(rustc) == CSD_RUSTC and sha(bin_dir/'cargo') == CSD_RCH_WRAPPER, 'not build656 toolchain'
    env = dict(PATH=str(bin_dir) + ':/usr/bin:/bin', HOME=str(Path.home()))
    version = text([cargo, '--version'], env)
    assert version == CARGO_VERSION, ('cargo version', version)
    verbose = text([rustc, '-vV'], env).splitlines()
    assert all(line in verbose for line in RUSTC_LINES), ('rustc identity', verbose)
    record = dict(bin_dir=str(bin_dir), cargo_is_rch_real=cargo == real, cargo_version=version, rustc_vv=verbose,
                  clippy_version=text([tools['clippy_driver'], '--version'], env),
                  sha256={k: sha(p) for k, p in tools.items()},
                  rch_wrapper_sha256=sha(bin_dir/'cargo') if cargo == real else None)
    return tools, record


def read_receipt(job):
    for _ in range(10):                                # the guard rewrites it every 10 s while we run
        try:
            return json.loads((job/'resource-receipt.json').read_text())
        except json.JSONDecodeError:
            time.sleep(0.5)
    raise SystemExit('731 refused: the guard receipt could not be read')


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
    assert os.environ.get('CARGO_TARGET_DIR') == str(TARGET), 'CARGO_TARGET_DIR is not the guard value'
    assert os.environ.get('RUSTUP_TOOLCHAIN') == TOOLCHAIN and os.environ.get('CARGO_BUILD_JOBS') == '2', 'guard env'
    assert os.environ.get('RAYON_NUM_THREADS') == '4', 'guard env'
    return dict(job=job.name, command=r['command'], policy=r['policy'], admission=r.get('admission'),
                inputs_sha256=r['inputs_sha256'])


def own_group():
    """The helper leads the group the guard made with start_new_session, so the group is this job and nothing else."""
    pid = os.getpid()
    assert os.getpgid(0) == pid and os.getsid(0) == pid, 'run 731 as the guard command itself, not under a wrapper'


def end_group(reason):
    """End the whole owned group, this helper last. Never returns."""
    for sig in STOP_SIGNALS:
        signal.signal(sig, signal.SIG_IGN)            # the helper outlives its own SIGTERM to write the record
    group = os.getpgid(0)
    os.killpg(group, signal.SIGTERM)                   # cargo, rustc, tests, the binary: all but this helper
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
    assert len(ORIGINAL_TESTS) == 30 == len(set(ORIGINAL_TESTS)) and not set(NEW_TESTS) & set(ORIGINAL_TESTS)


def bound_rows():
    rows, source, q656 = evidence656()
    rows.update(candidates())
    rows[str(HELPER)] = sha(HELPER)
    return rows, source, q656


def prepare_inputs(args):
    job = JOBS/args.job
    manifest = Q/(args.job + '-inputs.json')
    assert not job.exists() and not manifest.exists(), ('no duplicate', job, manifest)
    rows, _, _ = bound_rows()
    files = [dict(path=path, sha256=rows[path]) for path in sorted(rows)]
    check_files(files)
    previous = TARGET/'release'/BIN
    with manifest.open('x') as f:
        json.dump(dict(schema='apportionment-ch06-build731-inputs', job=args.job, files=files), f, indent=1)
    print(json.dumps(dict(manifest=str(manifest), sha256=sha(manifest), files=len(files),
                          target_binary_sha256=sha(previous) if previous.is_file() else None)))
    return 0


def run(args):
    own_group()
    for sig in STOP_SIGNALS:
        signal.signal(sig, on_stop)
    job = JOBS/args.job
    manifest = Q/(args.job + '-inputs.json')
    guard = guard_binding(job, manifest)
    STATE['job'] = job
    files = read(manifest)['files']
    check_files(files)
    want, source, q656 = bound_rows()
    have = {row['path']: row['sha256'] for row in files}
    assert want == have, ('the manifest does not bind exactly 656, the candidates and this helper',
                          sorted(set(want) ^ set(have))[:20])
    for k in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER'):
        assert k not in os.environ, k                  # 656's own check, kept word for word
    workspace, crate = prepare(job, source)
    os.sched_setaffinity(0, {0, 1, 2, 3})
    tools, tool_record = toolchain()
    home = cargo_home()
    problems, configs = config_problems(home, crate, [workspace/'.cargo/config.toml'])
    problems += env_problems()
    if problems:
        raise SystemExit('731 refused before Cargo: ' + '; '.join(problems))
    assert TARGET.is_dir() and not TARGET.is_symlink(), TARGET
    os.environ.update(PATH=str(tools['cargo'].parent) + ':/usr/bin:/bin', CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='2',
                      RCH_SHIM_LOCAL_IDE='1', CARGO_TARGET_DIR=str(TARGET))
    for folder in (TMP, CACHE):
        folder.mkdir(mode=0o700, exist_ok=True)
    os.environ.update(TMPDIR=str(TMP), XDG_CACHE_HOME=str(CACHE))
    built = TARGET/'release'/BIN
    previous = None
    if built.exists():                                 # build564's rule: keep the older binary before Cargo replaces it
        kept = job/(BIN + '-previous')
        shutil.copy2(built, kept)
        previous = sha(kept)
    keep = ('PATH', 'CARGO_HOME', 'CARGO_INCREMENTAL', 'CARGO_BUILD_JOBS', 'RCH_SHIM_LOCAL_IDE', 'CARGO_TARGET_DIR', 'TMPDIR',
            'XDG_CACHE_HOME', 'RAYON_NUM_THREADS', 'RUSTUP_TOOLCHAIN')
    preflight = dict(schema='apportionment-ch06-build731-preflight', node=platform.node(), guard=guard,
                     toolchain=tool_record, cargo_home=str(home), cargo_configs=configs, workspace=str(workspace),
                     crate=str(crate), workspace_listing=WORKSPACE731, crate_listing=CRATE731,
                     candidates={rel: after for rel, (_, _, after) in CANDIDATES.items()},
                     previous_binary_sha256=previous, previous_is_656=previous == q656['binary_sha256'],
                     python=platform.python_version(), env={k: os.environ.get(k) for k in keep})
    with (job/'preflight731.json').open('x') as f:
        json.dump(preflight, f, indent=2)

    def step(name, argv, seconds=STEP_SECONDS, extra=None):
        return run_step(job, crate, name, argv, seconds, extra)

    cargo = tools['cargo']
    common = ['--locked', '--offline', '--release', '--config', str(BUILD656/'pristine-engine.toml'),
              '--manifest-path', str(crate/'Cargo.toml')]
    one_bin = ['--bin', BIN]
    tests = step('tests', [cargo, 'test', *common, *one_bin], COLD_SECONDS).decode()
    counts = re.findall(RESULT, tests, re.M)
    assert counts == [str(len(ORIGINAL_TESTS) + len(NEW_TESTS))], tests[-2000:]
    passed = passed_tests(tests)
    assert sorted(passed) == sorted(ORIGINAL_TESTS + NEW_TESTS), ('passed names differ from 656 plus the three 722 tests',
                                                                  sorted(set(passed) ^ set(ORIGINAL_TESTS + NEW_TESTS)))
    step('clippy', [tools['cargo_clippy'], 'clippy', *common, '--all-targets', '--no-deps', '--', '-D', 'warnings'],
         extra=dict(CARGO=str(cargo)))
    step('build', [cargo, 'build', *common, *one_bin])
    binary = job/'apportionment-native731'
    assert not binary.exists()
    shutil.copy2(built, binary)
    assert native_binary(binary), binary

    facts = step('facts', [binary, 'facts'])
    oracle = step('oracle', [PYTHON, '-I', '-B', crate/'oracle.py'])
    native, independent = fact_rows(facts.decode()), fact_rows(oracle.decode())
    assert len(native) == FACTS and native == independent, sorted(
        k for k in set(native) | set(independent) if native.get(k) != independent.get(k))[:40]
    assert hashlib.sha256(oracle).hexdigest() == ORACLE_SHA, 'oracle.py no longer prints the pinned oracle output'
    assert oracle == (BUILD656/'oracle.stdout').read_bytes(), 'oracle output differs from 656'
    assert facts == (BUILD656/'facts.stdout').read_bytes(), 'native facts output differs from 656'
    checked = step('oracle-check', [PYTHON, '-I', '-B', crate/'oracle.py', '--check'])
    assert checked == oracle, 'oracle --check printed other facts'
    assert CENSUS_LINE in (job/'oracle-check.stderr').read_text().splitlines(), 'no census-year agreement line'

    chapters = step('chapters', [binary, 'chapters']).decode().splitlines()
    assert chapters == CHAPTERS == q656['chapters'], chapters
    rows = script_rows(crate)
    printed = step('script', [binary, 'script']).decode()
    assert printed == ''.join('%s\t%s\n' % row for row in rows), 'binary script differs from script.tsv'
    owned = check_script(rows, chapters)
    ids = [key for key, _ in rows]
    modules = static_cues(crate, ids)

    cue_out = job/'cue-probe'
    step('cue-probe', [binary, 'render', 'all', '--scratch-voice', '--res', '64x36', '--fps', '1', '--out', cue_out],
         extra=dict(PATH=str(Path(FFMPEG).parent) + ':/usr/bin:/bin'))
    said = said_cues(cue_out, chapters, owned, ids)
    glyph_out = job/'glyph-probe'
    glyph_text = step('glyphs', [binary, 'glyphs', glyph_out, '--res', '1920x1080']).decode()
    glyphs = check_glyphs(glyph_text, glyph_out, chapters)

    check_files(files)
    assert listing(workspace) == WORKSPACE731, 'the workspace Cargo read changed during the build'
    retained = [c for c in CHAPTERS if c != NEW_CHAPTER]
    provenance = dict(
        new=dict(chapter=NEW_CHAPTER, binary='apportionment-native731', binary_sha256=sha(binary),
                 candidates={rel: after for rel, (_, _, after) in CANDIDATES.items()},
                 note='Any chapter 06 render from repair722 must come from this binary. The old 06 final from '
                      'apportionment-native656 is preserved and is not this chapter.'),
        retained=dict(chapters=retained, binary_656_sha256=q656['binary_sha256'],
                      note='Their existing renders keep their own provenance: 01 and 02 are the local final632 '
                           'renders (export 682); 03-05 and 07-15 are CSD final672 renders from apportionment-native656. '
                           'This build does not show that this binary renders any of them the same.'),
        required_before_reuse=['rebind the 06_1 -> 06_2 prediction hold record: docs/narration-direction-map-frederick01.json '
                               'visual_holds[5] names not_every_time.rs at d3e88f7b, line 58; the candidate (04499441) has '
                               'the same marker, once, at line 102',
                               'equality of the 93 cue PCM recordings used', '45 normalized tables (15 chapters x '
                               'timeline, cues, cuelog) from this binary equal to the reference'])
    with (job/'qualification731.json').open('x') as out:
        json.dump(dict(schema='apportionment-ch06-build731', node=platform.node(), binary=str(binary),
                       binary_sha256=sha(binary), previous_binary_sha256=previous,
                       inputs_sha256=sha(manifest), input_files=len(files), preflight_sha256=sha(job/'preflight731.json'),
                       workspace=str(workspace), workspace_listing=WORKSPACE731, crate_listing=CRATE731,
                       source656=SOURCE656, toolchain_sha256=tool_record['sha256'],
                       tests_passed=int(counts[0]), original_tests_passed=len(ORIGINAL_TESTS), new_tests=list(NEW_TESTS),
                       tests_sha256=hashlib.sha256('\n'.join(sorted(passed)).encode()).hexdigest(), clippy_passed=True,
                       facts=len(native), facts_sha256=hashlib.sha256(facts).hexdigest(), facts_equal_656=True,
                       oracle_sha256=ORACLE_SHA, oracle_equal_656=True, census_line=CENSUS_LINE,
                       chapters=chapters, cues=ids, cues_per_chapter={k: len(v) for k, v in owned.items()},
                       cues_said_in_script_order=said == ids, chapter_modules=modules,
                       ffmpeg=dict(path=FFMPEG, sha256=sha(FFMPEG)), glyphs=glyphs, seconds=TIMES,
                       provenance=provenance,
                       scope='Native automated checks of the four repair722 candidates on the 656 closure only: tests, '
                             'Clippy, facts against the Python oracle and against 656, chapter and cue coverage, and a '
                             'glyph probe with zero failures whose pages no one has read. Cue timing is estimated. No '
                             'narration was generated or checked. No 06 hold rebind, PCM comparison, real-narration '
                             'timeline, frame, picture, motion, audio, fine-sync, AV or release acceptance, and no '
                             'reuse of any retained render.'), out, indent=2)
    print(json.dumps(dict(passed=True, job=job.name, binary_sha256=sha(binary))))
    return 0


def main():
    p = argparse.ArgumentParser(description=__doc__.split('\n', 1)[0])
    p.add_argument('mode', choices=('inputs', 'run'))
    p.add_argument('--job', required=True)
    args = p.parse_args()
    assert re.fullmatch(JOB_NAME, args.job), 'job must be csd-apportionment-ch06-build731 or that name plus -aN'
    check_host()
    return prepare_inputs(args) if args.mode == 'inputs' else run(args)


if __name__ == '__main__':
    raise SystemExit(main())
