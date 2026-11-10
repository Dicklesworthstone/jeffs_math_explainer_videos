"""Build798: test-only stage12 successor to preserved716; 15 chapters/94 cues/7008 words/218 authored tests.

Root source handoff773 supersedes the stage08 pins below. Historical716 descriptions and output schema names
are retained for compatibility with its diagnostics; this file's STAGE and final scope describe stage11.
No stage11 native build or audiovisual acceptance has been claimed. All host/resource/process checks remain.

Prepared only: it has not run on any host. Successor to build 697, which stays as it is. Root reports that guarded
csd-exponential-build697 ended exit 1 with stable inputs, that its 95 tests passed, and that a constant assertion was its
only Clippy finding.

The stage is expanded-stage08/native-crate, chapters 01 to 09, sealed by CC2's handoff712. The pins below were copied from
that handoff and recomputed from the sealed files. check_pins() still refuses both modes if any pin is None. This file is
bound to stage08 only: a later stage, such as one with the ending chapters, needs its own helper file and job names.
No stage08 Rust has been compiled, so a compile, test or Clippy stop is a plausible first result. A stop is a finding for
the source owner, who stages a successor. The successor's pins replace the STAGE block in a new file with new job names;
this file and the frozen crate stay as they are.

Steps, in order. Any failure stops the job and there is no retry:
  tests      cargo test --bin: exactly one result line, the pinned count passed, 0 failed, 0 ignored. Every name in
             ORIGINAL_TESTS must be among those that passed, so no test 697 ran can be dropped or renamed
  clippy     --all-targets --no-deps -D warnings, so test code is linted too, as in the earlier exponential builds
  build      the binary, copied under a new name into the job folder
  facts      binary facts equal oracle.py by key, at the pinned number of unique keys; oracle.py output has the pinned hash
  chapters   the 9 registered keys, in order
  script     binary script equals script.tsv: the pinned unique cues and words, each cue owned by one chapter, numbered in order
  static     every cue is said exactly once, as a literal, in a module the registry compiles (old unregistered files do not count)
  cue probe  render all --scratch-voice at 64x36, 1 fps. Each chapter's ids as really said, joined, must be the script order
  glyphs     the kit's probe: every chapter builds and registers strings, exit 0, empty failures.txt, registry and pages hashed

Hosts:
  csd  Linux, --release, guard_csd02 (10/8/4/56/20/10). Cargo works in a job-local copy of the crate; only its Cargo.toml
       path lines and its Cargo.lock engine source lines are edited, as build656 edited its copy. Kit, score and engine
       are used in place, read only, from finished build656, each at the hash 656's qualification.json recorded. No engine
       is copied or edited, and Cargo can reuse what earlier jobs compiled under D/work/target, so a later attempt is cheap.
       Finished 697's receipt and tests.stdout are also bound, read only: ORIGINAL_TESTS must be exactly what 697 passed.
  mac  Darwin arm64, debug, guard_owner80g_target20g (10/8/4/80/20/10). Cargo works in place, as every earlier exponential
       build did: the crate, the repo's crates/ and Cargo's git checkout of the pinned engine, all manifest-bound.
  mac-mini-max has no profile here: complex09's final is running there.

Process handling is build 521's: every step stays in the helper's own group, which the guard created. On a timeout, an
exception inside a step or a stop signal sent to the helper, end_group() ends that group. That does not include ffmpeg:
the pinned engine starts it in a process group of its own, in the same session. Only the cue probe reaches ffmpeg; the
glyph probe writes PNGs itself. The CSD guard's own stop covers ffmpeg. handoff717 states the limit on the helper's stops.

Usage, under the host's guard (see handoff717):  exponential-build716.py inputs|run --host csd|mac --job JOB --source DIR
`inputs` checks everything it can without Cargo and writes the guard's manifest. `run` is the guard's command.
Outputs, all new files in the job folder: preflight716.json, <step>.stdout/.stderr, any <step>.cleanup.json, the binary,
cue-probe/, glyph-probe/, qualification.json; on csd also workspace/. Cargo writes only to its target folder.

It authorizes no render beyond its own 64x36 cue probe, which uses estimated timing and no recordings. It generates no
narration and checks none (current narration: ElevenLabs Frederick Surrey). It accepts no picture, motion, audio, fine
sync, AV or release result. Zero glyph failures can hide fallback faces, so the pages still need reading.
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

# CC2's sealed handoff712, each value recomputed from the sealed files; check_pins() refuses both modes if one is None.
# The tree hash is sha256 of the sorted "<sha256>  <path>\n" lines over the crate's files.
STAGE = dict(tree_sha256='c5945a1a630ab5ad0887e983679d103a6b3c48d95077393066b99325851524c1', files=55,
             handoff_sha256='21923623ccc12e4229d2977e7ebb8b724788319220c030e145a6a52927049f7a',
             lock_sha256='60338289fe9224d5b8321557cce27d075326fa8d467bf4ed2b12dcb23aa9139c',
             oracle_sha256='123eef033530a2330951c6c5f1852c6d5199cec1d6021896f3e8c14749c35630',
             package='exponential_magic', binary='exponential_magic_expanded_stage11',
             tests=218, facts=1771, cues=94, words=7008,
             chapters=['01_rate_law', '02_factorials', '03_product_grid', '04_tiny_multipliers', '05_whole_sum',
                       '06_only_solution', '07_calculating_ruler', '08_decay_law', '09_charging_gap',
                       '10_changing_rates', '11_money_limits', '12_chance_and_limits', '13_turning_rate',
                       '14_euler_identity', '15_logarithm_bridge'])

# Every test build 697 compiled and ran, from expanded-stage06 (tree af107280...). Each must pass here too.
ORIGINAL_TESTS = tuple(sorted("""
chapters::factorials::tests::each_equation_lands_on_its_sentence_and_is_then_solved
chapters::factorials::tests::every_clause_is_in_its_line_in_spoken_order
chapters::factorials::tests::live_bars_lose_their_updaters_before_they_fade
chapters::factorials::tests::no_factorial_sign_appears_before_the_sentence_that_names_them
chapters::factorials::tests::the_chapter_ends_with_all_three_debts_in_payment_order
chapters::factorials::tests::the_dots_are_on_screen_before_they_are_refused
chapters::factorials::tests::the_half_waits_for_eventually
chapters::factorials::tests::the_hypotheses_are_printed_where_they_are_used
chapters::factorials::tests::the_polynomial_lands_on_try_a_polynomial
chapters::factorials::tests::the_proof_steps_land_on_their_sentences
chapters::factorials::tests::the_recurrence_does_not_reuse_the_tables_letter
chapters::factorials::tests::the_think_gap_is_authored_and_the_answer_waits
chapters::only_solution::tests::every_phrase_hook_is_in_its_own_line
chapters::only_solution::tests::the_assumptions_are_on_screen_and_the_model_question_stays_open
chapters::only_solution::tests::the_cancellation_is_shown_before_it_is_used
chapters::only_solution::tests::the_cues_run_in_film_order
chapters::only_solution::tests::the_debts_keep_the_words_the_series_chapter_wrote
chapters::only_solution::tests::the_result_enters_the_ledger_last_and_nothing_runs_back_from_it
chapters::only_solution::tests::the_test_is_built_for_any_y_before_the_rule_is_applied
chapters::only_solution::tests::the_think_gap_is_authored_and_the_answer_waits
chapters::product_grid::tests::each_diagonal_and_its_total_share_a_colour_on_the_key_sentence
chapters::product_grid::tests::every_phrase_is_in_its_line_in_spoken_order
chapters::product_grid::tests::the_change_of_reading_is_seen_before_degree_zero_lands
chapters::product_grid::tests::the_name_is_earned_in_the_order_the_voice_earns_it
chapters::product_grid::tests::the_proof_steps_land_on_their_sentences
chapters::product_grid::tests::the_think_gap_is_authored_and_the_answer_waits
chapters::rate_law::tests::each_reveal_waits_for_the_sentence_that_names_it
chapters::rate_law::tests::no_refinement_runs_under_the_old_slope_or_its_read_out
chapters::rate_law::tests::the_think_gap_is_authored_and_nothing_new_appears_during_it
chapters::tiny_multipliers::tests::e_is_named_as_a_special_case_not_the_general_formula
chapters::tiny_multipliers::tests::every_phrase_hook_is_in_its_own_line
chapters::tiny_multipliers::tests::live_bars_lose_their_updaters_before_they_fade
chapters::tiny_multipliers::tests::minus_three_prints_every_kind_of_base
chapters::tiny_multipliers::tests::outlines_and_open_paths_are_never_recoloured_with_a_fill
chapters::tiny_multipliers::tests::the_cues_run_in_film_order_each_from_its_own_function
chapters::tiny_multipliers::tests::the_cut_is_chosen_before_n_moves
chapters::tiny_multipliers::tests::the_debt_is_paid_last_in_its_own_words
chapters::tiny_multipliers::tests::the_factorial_is_earned_by_the_double_count_before_the_chain_appears
chapters::tiny_multipliers::tests::the_multiplier_is_earned_before_the_power_is_written
chapters::tiny_multipliers::tests::the_negative_input_uses_the_same_ceilings_and_is_measured
chapters::tiny_multipliers::tests::the_series_comparison_waits_for_the_counted_coefficient
chapters::tiny_multipliers::tests::the_think_gap_is_authored_and_the_answer_waits
chapters::tiny_multipliers::tests::the_two_chapters_split_where_the_teaching_turns
chapters::tiny_multipliers::tests::the_warning_is_seen_before_the_ceilings_rescue_it
facts::tests::finite_examples_are_below_exp_one
facts::tests::growth_and_slide_examples_are_consistent
limit_model::tests::every_chain_stays_in_range_and_empties_past_n
limit_model::tests::no_bar_passes_its_ceiling_at_either_sign
limit_model::tests::the_chain_is_the_binomial_coefficient_shared_out
limit_model::tests::the_distance_is_under_the_total_of_the_gaps
limit_model::tests::the_drawn_count_shows_gaps_that_the_bound_count_closes
limit_model::tests::the_edge_table_shows_each_kind_of_base_and_is_not_monotone
limit_model::tests::the_expansion_adds_up_to_the_product_at_both_signs_and_every_base
limit_model::tests::the_far_column_is_still_nearly_empty_at_every_drawn_count
limit_model::tests::the_printed_numbers_tell_the_story_of_the_cut
limit_model::tests::the_rate_one_table_climbs_by_less_and_less_and_stays_below_e
limit_model::tests::the_strip_counts_are_pascals_row_and_the_choices_list_them
limit_model::tests::the_walks_end_at_the_printed_endpoints
limit_model::tests::the_warning_sum_stays_at_one_while_every_fixed_column_falls
limit_model::tests::the_zero_input_is_the_trivial_edge
phrase::tests::a_mark_on_another_word_is_not_used
phrase::tests::a_narrator_that_splits_the_line_differently_is_an_error
phrase::tests::a_phrase_the_line_lacks_is_an_error
phrase::tests::a_word_mark_wins_over_the_share
phrase::tests::without_a_mark_the_phrase_takes_its_share_of_its_own_sentence
product_model::tests::diagonals_have_every_term_once_and_no_truncated_degree
product_model::tests::every_printed_cell_is_exact
product_model::tests::factorial_weights_are_exact_binomial_coefficients
product_model::tests::the_line_marks_every_step_once_and_in_its_own_group
product_model::tests::the_named_constant_is_e
rate_model::tests::displayed_refinements_stay_on_the_same_domain_and_improve_endpoint
rate_model::tests::each_refinement_climbs_above_the_coarser_drawing_and_the_end_gap_shrinks
rate_model::tests::every_printed_number_is_the_exact_rational_rounded
rate_model::tests::first_step_agrees_but_the_next_slope_does_not
series_model::tests::bound_contains_positive_and_negative_sample_errors
series_model::tests::derivative_coefficients_match_the_previous_power
series_model::tests::every_ratio_is_the_exact_step_between_two_bars
series_model::tests::every_tail_bar_sits_under_its_ceiling_at_every_input
series_model::tests::the_derivative_of_each_term_is_the_previous_term
series_model::tests::the_hypothesis_is_exact_at_its_edge
series_model::tests::the_printed_bounds_are_rounded_up_and_still_contain_the_true_tail
series_model::tests::the_running_products_are_the_factorials
series_model::tests::the_table_prints_the_exact_equations
slide_model::tests::alignments_and_decade_are_exact_within_float_precision
slide_model::tests::equal_ratios_have_equal_lengths_and_spacing_contracts
slide_model::tests::every_whole_number_is_labelled_and_the_labelled_gaps_shrink
slide_model::tests::off_scale_is_not_a_tick_and_division_inverts_alignment
solution_model::tests::both_pieces_obey_the_rule_and_the_far_piece_is_not_e
solution_model::tests::e_itself_reads_level_at_one
solution_model::tests::each_rate_solves_its_law_and_the_edges_behave
solution_model::tests::every_curve_fits_under_its_panel_top
solution_model::tests::the_imagined_curve_starts_at_one_and_is_not_a_solution
solution_model::tests::the_polynomials_start_at_one_and_tilt_down_with_the_higher_degree_flatter
solution_model::tests::the_written_derivative_matches_the_slope_numerically
solution_model::tests::the_written_slope_is_exactly_minus_the_top_term_times_e_of_minus_x
""".split()))

# The files this helper's parsing relies on, and the workspace files Cargo reads. All match the 652/655 closure.
KIT_PINS = {'crates/explainer_kit/Cargo.toml': '0fc78621600eafca2649b603dc952ecc966c267b8f19f8975b0a91e7b82d324b',
            'crates/explainer_kit/src/glyphs.rs': '8652dd294c7416e577d4c4a2df821d3f528fda09b7b4185c8f0d2a7db0bf3d2d',
            'crates/explainer_kit/src/cli.rs': 'a63970e0120e2ae2ce5c374b4593cb2ad45011696b0112ac17e801e110640d4a',
            'crates/explainer_kit/src/narration.rs': '8072720d8ff86b91ad35d6904e148d330fdcf736b931f393f12b8bc9e4f823b8',
            'crates/explainer_kit/src/kit.rs': 'ef035faf236498e7485fb60a66d862b383e1a5040fa17962e1633596ed99e106',
            'crates/explainer_score/Cargo.toml': '4f2bcfe9bfdd6aa0c0003f5a32280684929101604b83ad65149aae188790537e',
            '.cargo/config.toml': '79fb8766d1f41365d64066e5b2e68b0fb89c50015425d109b155ccaf8eff2716',
            'rust-toolchain.toml': '23cbc5a95ccef09cfa4ca1e5dbef11129eb439b012f1061d016a0682ad2075c5'}
REPO_WORKSPACE_TOML = '269714e2695567f3062c05c150cae6bc6e3e438297176d31bf218df553decce7'
BUILD656_WORKSPACE_TOML = '2fa23046569fa50378540c709509e12943a73733682564acdaa394541c7e2d05'   # members = ["crates/*"]

CSD_ROOT = Path('/data/projects/jmev-stroke-probe-20261106')
BUILD656 = CSD_ROOT/'jobs/csd-apportionment-build656'
BUILD697 = CSD_ROOT/'jobs/csd-exponential-build697'
TOOLCHAIN = 'nightly-2026-08-31'
CARGO_VERSION = 'cargo 1.100.0-nightly (e8cb624d5 2026-08-22)'
RUSTC_LINES = ('release: 1.100.0-nightly', 'commit-hash: 90850177249efe0321573c569aec5d12b257f8d6',
               'commit-date: 2026-08-30')
CSD_RCH_WRAPPER = 'b854c9424332416075760b8b5fc8a43254e8eea8f38f14a57deea6126810f1b9'   # build656's two toolchain pins
CSD_RUSTC = '49f9fa06f3e2b93aba85273433fef67b984ddb4cacb4500de56e85f70ec2e339'
ENGINE_URL = 'https://github.com/Dicklesworthstone/franken_manim'
ENGINE_SOURCE = 'git+' + ENGINE_URL + '?'
ENGINE_LOCK_LINE = (r'^source = "git\+https://github\.com/Dicklesworthstone/franken_manim\?'
                    r'rev=a399d626c43ecc852a5c795288979f52b446bc5c#a399d626c43ecc852a5c795288979f52b446bc5c"\n')
ENGINE_PACKAGES = 19
ENGINE_CHECKOUT = 'git/checkouts/franken_manim-6641d0e09642f63e/a399d62'
FORBIDDEN_ENV = ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER',
                 'RUSTDOC', 'RUSTDOCFLAGS', 'CARGO', 'CARGO_BUILD_TARGET')
FORBIDDEN_ENV_PREFIXES = ('CARGO_BUILD_RUSTC', 'CARGO_BUILD_RUSTFLAGS', 'CARGO_PROFILE_', 'CARGO_UNSTABLE_',
                          'CARGO_TARGET_', 'CARGO_ENV_', 'CARGO_PATCH', 'CARGO_SOURCE')
FORBIDDEN_TABLES = ('target', 'patch', 'source', 'env', 'unstable', 'replace')
FORBIDDEN_BUILD_KEYS = ('rustflags', 'rustdocflags', 'rustc', 'rustc-wrapper', 'rustc-workspace-wrapper',
                        'rustdoc', 'target', 'incremental')
ELF = b'\x7fELF'
ARM64_MACHO = b'\xcf\xfa\xed\xfe\x0c\x00\x00\x01'
STEP_SECONDS = 7200
COLD_SECONDS = 14400                                   # the first Cargo step may build cold
TERM_GRACE = 6                                         # the guard waits 20 s after its SIGTERM, then SIGKILL
STOP_SIGNALS = (signal.SIGTERM, signal.SIGINT, signal.SIGHUP)
HELPER = Path(__file__).resolve()
STATE = dict(job=None, step=None, child=None)          # for end_group()'s record only; stopping never needs it
TIMES = {}


def host_profile(name):
    """Everything that differs between hosts. Mac paths are derived here, and only when the Mac is chosen."""
    policy = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4, work_cap_gib=56, target_growth_cap_gib=20,
                  sample_seconds=10)
    if name == 'csd':
        work = CSD_ROOT/'work'
        return dict(name=name, system='Linux', machine='x86_64', jobs=CSD_ROOT/'jobs',
                    manifests=CSD_ROOT/'qualification01/production-repair17', guard_target=work, target=work/'target',
                    tmp=work/'tmp', cache=work/'cache', python='/usr/bin/python3', ffmpeg='/usr/bin/ffmpeg',
                    profile='release', triple='x86_64-unknown-linux-gnu', magic=ELF, policy=policy)
    repo = HELPER.parents[7]
    assert (repo/'crates/explainer_kit/Cargo.toml').is_file(), repo
    work = repo/'narration_lab/third-wave-20261024/root/five-film-release-20261027'
    return dict(name=name, system='Darwin', machine='arm64', jobs=work, manifests=work, guard_target=repo/'target',
                target=repo/'target', tmp=None, cache=None, python='/opt/homebrew/bin/python3',
                ffmpeg='/opt/homebrew/bin/ffmpeg', profile='debug', triple='aarch64-apple-darwin', magic=ARM64_MACHO,
                policy=dict(policy, work_cap_gib=80), repo=repo)


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()


def read(path):
    return json.loads(Path(path).read_text())


def listing(top):
    """sha256 of sorted "<sha256>  <path>\\n" over every file; no links, no specials. CC1's and CC0's 492/506 rule."""
    rows = []
    for parent, dirs, files in os.walk(top, followlinks=False):
        for name in dirs + files:
            path = Path(parent, name)
            if path.is_symlink():
                raise SystemExit('716 refused: symlink in the tree: %s' % path)
        for name in files:
            path = Path(parent, name)
            if not path.is_file():
                raise SystemExit('716 refused: not a regular file: %s' % path)
            rows.append((path.relative_to(top).as_posix(), sha(path)))
    rows.sort()
    text = ''.join('%s  %s\n' % (digest, rel) for rel, digest in rows)
    return len(rows), hashlib.sha256(text.encode()).hexdigest()


def files_under(top):
    return [Path(parent, name) for parent, _, names in os.walk(top, followlinks=False) for name in names]


def check_files(rows):
    for row in rows:
        assert sha(row['path']) == row['sha256'], ('input changed', row['path'])


def engine_packages(lock_text):
    engine = [p for p in tomllib.loads(lock_text)['package'] if p.get('source', '').startswith(ENGINE_SOURCE)]
    assert len(engine) == ENGINE_PACKAGES, len(engine)
    return engine


def engine_patch(lock_text, engine):
    """build656's pristine-engine.toml, byte for byte, for an engine folder."""
    out = '[patch."%s"]\n' % ENGINE_URL
    for package in engine_packages(lock_text):
        out += package['name'] + ' = { path = ' + json.dumps(str(engine/'crates'/package['name'])) + ' }\n'
    return out


def passed_tests(text):
    """The names libtest reported as passing, one per line; a repeated name is refused."""
    names = re.findall(r'^test (\S+)(?: - should panic)? \.\.\. ok$', text, re.M)
    assert len(set(names)) == len(names), 'a test name repeats'
    return names


def build697_evidence():
    """CSD only. Finished 697's receipt and test log, read only: ORIGINAL_TESTS is exactly what it really passed."""
    receipt, log = BUILD697/'resource-receipt.json', BUILD697/'tests.stdout'
    r = read(receipt)
    assert r.get('admitted') is True and r.get('finished') and r.get('exit') == 1, ('697 receipt', r.get('exit'))
    assert r.get('inputs_sha256') == r.get('final_inputs_sha256'), '697 inputs changed'
    assert not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), '697 receipt'
    names = passed_tests(log.read_text())
    assert sorted(names) == list(ORIGINAL_TESTS), ('697 passed another list', sorted(set(names) ^ set(ORIGINAL_TESTS))[:40])
    return {str(path): sha(path) for path in (receipt, log)}


def csd_closure(lock_text):
    """Finished build656's kit, score and engine at the hashes it recorded. Returned rows are verified, never written."""
    qualification, receipt = BUILD656/'qualification.json', BUILD656/'resource-receipt.json'
    r = read(receipt)
    assert r.get('admitted') is True and r.get('finished') and r.get('exit') == 0, ('656 receipt', r.get('exit'))
    assert r.get('inputs_sha256') == r.get('final_inputs_sha256'), '656 inputs changed'
    assert not any(r.get(k) for k in ('error', 'resource_stop', 'stopped_for_resource_limit')), '656 receipt'
    keep = ('engine/', 'workspace/crates/explainer_kit/', 'workspace/crates/explainer_score/')
    tops = {'workspace/' + n for n in ('Cargo.toml', 'rust-toolchain.toml', 'rustfmt.toml', 'LICENSE', '.cargo/config.toml')}
    rows = {str(BUILD656/rel): digest for rel, digest in read(qualification)['source'].items()
            if rel.startswith(keep) or rel in tops}
    for rel, digest in dict(KIT_PINS, **{'Cargo.toml': BUILD656_WORKSPACE_TOML}).items():
        assert rows.get(str(BUILD656/'workspace'/rel)) == digest, ('656 closure differs', rel)
    config = BUILD656/'pristine-engine.toml'
    assert config.read_text() == engine_patch(lock_text, BUILD656/'engine'), '656 patch file differs'
    for path in (qualification, receipt, config):
        rows[str(path)] = sha(path)
    rows.update(build697_evidence())
    return rows


def mac_closure(P, lock_text, home):
    """The repo's kit and score, its workspace files and Cargo's checkout of the pinned engine, at today's hashes."""
    repo = P['repo']
    paths = [repo/n for n in ('Cargo.toml', 'rust-toolchain.toml', 'rustfmt.toml', 'LICENSE', '.cargo/config.toml')]
    paths += files_under(repo/'crates/explainer_kit') + files_under(repo/'crates/explainer_score')
    engine = home/ENGINE_CHECKOUT
    paths.append(engine/'Cargo.toml')
    for package in engine_packages(lock_text):
        paths += files_under(engine/'crates'/package['name'])
    rows = {str(p): sha(p) for p in paths}
    for rel, digest in dict(KIT_PINS, **{'Cargo.toml': REPO_WORKSPACE_TOML}).items():
        assert rows.get(str(repo/rel)) == digest, ('repo closure differs', rel)
    return rows


def closure(P, lock_text, home):
    return csd_closure(lock_text) if P['name'] == 'csd' else mac_closure(P, lock_text, home)


def rewrite_manifest(text, crates):
    """Point the two path dependencies at `crates`; nothing else in Cargo.toml may change."""
    data = tomllib.loads(text)
    expected = json.loads(json.dumps(data))
    for name in ('explainer_kit', 'explainer_score'):
        old = data['dependencies'][name]['path']
        assert old.endswith('/crates/' + name) and text.count('"%s"' % old) == 1, old
        text = text.replace('"%s"' % old, json.dumps(str(crates/name)))
        expected['dependencies'][name]['path'] = str(crates/name)
    assert tomllib.loads(text) == expected, 'Cargo.toml edit went beyond the two paths'
    return text


def strip_engine_sources(text):
    """build656's lock edit: patched engine packages carry no git source line; nothing else may change."""
    data = tomllib.loads(text)
    engine_packages(text)
    expected = json.loads(json.dumps(data))
    for package in expected['package']:
        if package.get('source', '').startswith(ENGINE_SOURCE):
            del package['source']
    new = re.sub(ENGINE_LOCK_LINE, '', text, flags=re.M)
    assert tomllib.loads(new) == expected, 'Cargo.lock edit went beyond the engine source lines'
    return new


def prepare_csd(job, source):
    """The job's own copy of the crate, placed as a topic beside a .cargo/config.toml. The source is not touched."""
    workspace = job/'workspace'
    crate = workspace/'explainer_corpus'/STAGE['package']
    for path in files_under(source):
        out = crate/path.relative_to(source)
        out.parent.mkdir(parents=True, exist_ok=True)
        with path.open('rb') as src, out.open('xb') as dst:
            shutil.copyfileobj(src, dst)
    config = workspace/'.cargo/config.toml'
    config.parent.mkdir()
    with (BUILD656/'workspace/.cargo/config.toml').open('rb') as src, config.open('xb') as dst:
        shutil.copyfileobj(src, dst)
    edits = {}
    for name, edit in (('Cargo.toml', lambda t: rewrite_manifest(t, BUILD656/'workspace/crates')),
                       ('Cargo.lock', strip_engine_sources)):
        path = crate/name
        before = sha(path)
        path.write_text(edit(path.read_text()))
        edits[name] = dict(before=before, after=sha(path))
    return crate, edits, [config]


def check_in_place(P, source):
    data = tomllib.loads((source/'Cargo.toml').read_text())
    for name in ('explainer_kit', 'explainer_score'):
        assert data['dependencies'][name]['path'] == str(P['repo']/'crates'/name), name


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
    assert len(ids) == STAGE['cues'] and len(set(ids)) == len(ids), len(ids)
    assert sum(len(text.split()) for _, text in rows) == STAGE['words'], 'word count'
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
    """Each cue said exactly once, by a literal, outside test code, in a module chapters/mod.rs compiles."""
    chapters = crate/'src/chapters'
    said, modules = [], re.findall(r'^(?:pub )?mod (\w+);', (chapters/'mod.rs').read_text(), re.M)
    for module in modules:
        text = (chapters/(module + '.rs')).read_text().split('\n#[cfg(test)]\nmod tests {')[0]
        assert not re.findall(r'\bsay\(\s*[A-Za-z_&.]+\s*,\s*[^"\s]', text), ('a cue that is not a literal', module)
        said += re.findall(r'\bsay\(\s*[A-Za-z_&.]+\s*,\s*"([0-9]{2}_[0-9]+)"', text)
    assert sorted(said) == sorted(ids), ('static cue coverage', sorted(set(ids) ^ set(said)), len(said))
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


def fact_rows(text):
    pairs = [line.split('\t', 1) for line in text.splitlines() if line.strip()]
    result = dict(pairs)
    assert len(result) == len(pairs), 'duplicate fact keys'
    return result


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
    pages = {p.relative_to(out).as_posix(): sha(p) for p in sorted(files_under(out/'pages'))}
    assert pages, 'no probe pages'
    return dict(sources=int(tail[0][1]), per_chapter={m[1]: int(m[2]) for m in registered}, failures=0,
                registry_sha256=sha(out/'registry.tsv'), pages=pages)


def cargo_home():
    home = Path(os.environ.get('CARGO_HOME') or Path.home()/'.cargo')
    assert home.is_absolute(), home
    return home


def config_problems(home, crate, bound):
    """Only the pinned workspace config and a harmless CARGO_HOME config may reach Cargo.

    The owner's Mac config sets [profile.dev] and [profile.test], as it did for every earlier Mac build. Those two are
    recorded in the preflight and allowed; any other profile table is refused."""
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
                elif sha(path) != KIT_PINS['.cargo/config.toml']:
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


def native_binary(path, magic):
    with Path(path).open('rb') as f:
        return f.read(len(magic)) == magic


def toolchain(P):
    """Never a shim: cargo-rch-real when RCH has replaced bin/cargo. Same nightly as 656 and 521, by version and commit."""
    rustup = Path.home()/'.cargo/bin/rustup'
    rustc = Path(subprocess.check_output([str(rustup), 'which', '--toolchain', TOOLCHAIN, 'rustc'], text=True,
                                         timeout=120).strip())
    bin_dir = rustc.parent
    assert bin_dir.parent.name == TOOLCHAIN + '-' + P['triple'], ('wrong toolchain', bin_dir)
    real = bin_dir/'cargo-rch-real'
    cargo = real if real.is_file() else bin_dir/'cargo'
    tools = dict(cargo=cargo, rustc=rustc, cargo_clippy=bin_dir/'cargo-clippy', clippy_driver=bin_dir/'clippy-driver')
    for name, path in tools.items():
        assert path.is_file() and native_binary(path, P['magic']), ('%s is not a native binary (a shim?)' % name, path)
    if P['name'] == 'csd':
        assert sha(rustc) == CSD_RUSTC and sha(bin_dir/'cargo') == CSD_RCH_WRAPPER, 'not build656 toolchain'
    env = dict(PATH=str(bin_dir) + ':/usr/bin:/bin', HOME=str(Path.home()))
    version = text([cargo, '--version'], env)
    assert version == CARGO_VERSION, ('cargo version', version)
    verbose = text([rustc, '-vV'], env).splitlines()
    assert all(line in verbose for line in RUSTC_LINES + ('host: ' + P['triple'],)), ('rustc identity', verbose)
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
    raise SystemExit('716 refused: the guard receipt could not be read')


def guard_binding(P, job, manifest):
    assert job.parent == P['jobs'] and job.is_dir() and not job.is_symlink(), job
    present = sorted(p.name for p in job.iterdir())
    assert present == ['command.log', 'resource-receipt.json'], ('the job folder is not fresh', present)
    r = read_receipt(job)
    assert r['admitted'] is True and 'exit' not in r and 'finished' not in r, 'receipt state'
    assert r['phase'] == 'native' and r['preflight_only'] is False, 'receipt phase'
    assert r['policy'] == P['policy'], ('policy', r['policy'])
    assert r['command'] == [P['python'], '-I', '-B', str(HELPER)] + sys.argv[1:], ('command', r['command'])
    assert r['inputs_sha256'] == sha(manifest), 'the guard bound another manifest'
    assert os.environ.get('CARGO_TARGET_DIR') == str(P['guard_target']), 'CARGO_TARGET_DIR is not the guard value'
    assert os.environ.get('RUSTUP_TOOLCHAIN') == TOOLCHAIN and os.environ.get('CARGO_BUILD_JOBS') == '2', 'guard env'
    assert os.environ.get('RAYON_NUM_THREADS') == '4', 'guard env'
    return dict(job=job.name, command=r['command'], policy=r['policy'], admission=r.get('admission'),
                inputs_sha256=r['inputs_sha256'])


def own_group():
    """The helper leads the group the guard made with start_new_session, so the group is this job and nothing else."""
    pid = os.getpid()
    assert os.getpgid(0) == pid and os.getsid(0) == pid, 'run 716 as the guard command itself, not under a wrapper'


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


def checked_source(P, value):
    source = Path(value)
    assert source.is_absolute() and os.path.realpath(source) == str(source), ('--source must be a real absolute path', value)
    assert listing(source) == (STAGE['files'], STAGE['tree_sha256']), 'source is not the frozen stage tree'
    assert sha(source/'Cargo.lock') == STAGE['lock_sha256'], 'Cargo.lock'
    return source


def check_pins():
    """Fail closed: no mode does anything until the sealed handoff's values are in STAGE."""
    unset = sorted(key for key, value in STAGE.items() if value is None)
    assert not unset, '716 refused: expanded-stage08 is not sealed; unset pins: %s' % ', '.join(unset)
    assert len(ORIGINAL_TESTS) == 95 == len(set(ORIGINAL_TESTS)), len(ORIGINAL_TESTS)


def check_host(P):
    assert sys.version_info >= (3, 11) and sys.flags.isolated and sys.dont_write_bytecode, 'run with python3 -I -B'
    assert platform.system() == P['system'] and platform.machine() == P['machine'], 'this host is not the chosen profile'


def prepare_inputs(args, P):
    job = P['jobs']/args.job
    manifest = P['manifests']/(args.job + '-inputs.json')
    assert not job.exists() and not manifest.exists(), ('no duplicate', job, manifest)
    source = checked_source(P, args.source)
    if P['name'] == 'mac':
        check_in_place(P, source)
    rows = closure(P, (source/'Cargo.lock').read_text(), cargo_home())
    rows.update({str(p): sha(p) for p in files_under(source)})
    rows[str(HELPER)] = sha(HELPER)
    handoff = HELPER.parent/'root-exponential797-source-handoff.json'
    assert sha(handoff) == STAGE['handoff_sha256']
    rows[str(handoff)] = STAGE['handoff_sha256']
    files = [dict(path=path, sha256=rows[path]) for path in sorted(rows)]
    check_files(files)
    with manifest.open('x') as f:
        json.dump(dict(schema='exponential-build716-inputs', host=P['name'], source=str(source), files=files), f, indent=1)
    print(json.dumps(dict(manifest=str(manifest), sha256=sha(manifest), files=len(files))))
    return 0


def run(args, P):
    own_group()
    for sig in STOP_SIGNALS:
        signal.signal(sig, on_stop)
    job = P['jobs']/args.job
    manifest = P['manifests']/(args.job + '-inputs.json')
    guard = guard_binding(P, job, manifest)
    STATE['job'] = job
    files = read(manifest)['files']
    check_files(files)
    source = checked_source(P, args.source)
    assert str(HELPER) in {row['path'] for row in files} and str(source/'Cargo.lock') in {row['path'] for row in files}
    lock_text = (source/'Cargo.lock').read_text()
    home = cargo_home()
    want = closure(P, lock_text, home)
    have = {row['path']: row['sha256'] for row in files}
    assert all(have.get(path) == digest for path, digest in want.items()), 'closure differs from the manifest'
    for k in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS', 'RUSTC', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER'):
        assert k not in os.environ, k                  # 656's own check, kept word for word
    config_arg = []
    if P['name'] == 'csd':
        crate, edits, bound = prepare_csd(job, source)
        config_arg = ['--config', str(BUILD656/'pristine-engine.toml')]
        os.sched_setaffinity(0, {0, 1, 2, 3})
    else:
        check_in_place(P, source)
        crate, edits, bound = source, {}, [P['repo']/'.cargo/config.toml']
    crate_listing = listing(crate)
    tools, tool_record = toolchain(P)
    problems, configs = config_problems(home, crate, bound)
    problems += env_problems()
    if problems:
        raise SystemExit('716 refused before Cargo: ' + '; '.join(problems))
    assert P['target'].is_dir() and not P['target'].is_symlink(), P['target']
    os.environ.update(PATH=str(tools['cargo'].parent) + ':/usr/bin:/bin', CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='2',
                      RCH_SHIM_LOCAL_IDE='1', CARGO_TARGET_DIR=str(P['target']))
    if P['name'] == 'csd':
        for folder in (P['tmp'], P['cache']):
            folder.mkdir(mode=0o700, exist_ok=True)
        os.environ.update(TMPDIR=str(P['tmp']), XDG_CACHE_HOME=str(P['cache']))
    else:
        os.environ['RUST_TEST_THREADS'] = '4'
    built = P['target']/P['profile']/STAGE['binary']
    previous = None
    if built.exists():                                 # build564's rule: keep any older binary before Cargo replaces it
        kept = job/(STAGE['binary'] + '-previous')
        shutil.copy2(built, kept)
        previous = sha(kept)
    keep = ('PATH', 'CARGO_HOME', 'CARGO_INCREMENTAL', 'CARGO_BUILD_JOBS', 'RCH_SHIM_LOCAL_IDE', 'CARGO_TARGET_DIR', 'TMPDIR',
            'XDG_CACHE_HOME', 'RUST_TEST_THREADS', 'RAYON_NUM_THREADS', 'RUSTUP_TOOLCHAIN')
    preflight = dict(schema='exponential-build716-preflight', host=P['name'], node=platform.node(), profile=P['profile'],
                     guard=guard, toolchain=tool_record, cargo_home=str(home), cargo_configs=configs, crate=str(crate),
                     source=str(source), source_tree_sha256=STAGE['tree_sha256'], crate_listing=crate_listing,
                     job_copy_edits=edits, previous_binary_sha256=previous, python=platform.python_version(),
                     env={k: os.environ.get(k) for k in keep})
    with (job/'preflight716.json').open('x') as f:
        json.dump(preflight, f, indent=2)

    def step(name, argv, seconds=STEP_SECONDS, extra=None):
        return run_step(job, crate, name, argv, seconds, extra)

    cargo = tools['cargo']
    common = ['--locked', '--offline'] + (['--release'] if P['profile'] == 'release' else []) + config_arg
    common += ['--manifest-path', str(crate/'Cargo.toml')]
    one_bin = ['--bin', STAGE['binary']]
    tests = step('tests', [cargo, 'test', *common, *one_bin], COLD_SECONDS).decode()
    counts = re.findall(r'^test result: ok\. (\d+) passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;', tests, re.M)
    assert counts == [str(STAGE['tests'])], tests[-2000:]
    passed = passed_tests(tests)
    assert len(passed) == STAGE['tests'], ('named test lines', len(passed))
    lost = sorted(set(ORIGINAL_TESTS) - set(passed))
    assert not lost, ('an original test did not pass', lost[:40])
    step('clippy', [tools['cargo_clippy'], 'clippy', *common, '--all-targets', '--no-deps', '--', '-D', 'warnings'],
         extra=dict(CARGO=str(cargo)))
    step('build', [cargo, 'build', *common, *one_bin])
    binary = job/(STAGE['binary'] + '-798')
    assert not binary.exists()
    shutil.copy2(built, binary)
    assert native_binary(binary, P['magic']), binary

    facts = step('facts', [binary, 'facts'])
    oracle = step('oracle', [P['python'], '-I', '-B', crate/'oracle.py'])
    assert hashlib.sha256(oracle).hexdigest() == STAGE['oracle_sha256'], 'oracle.py no longer prints the pinned oracle output'
    native, independent = fact_rows(facts.decode()), fact_rows(oracle.decode())
    assert len(native) == STAGE['facts'] and native == independent, sorted(
        k for k in set(native) | set(independent) if native.get(k) != independent.get(k))[:40]

    chapters = step('chapters', [binary, 'chapters']).decode().splitlines()
    assert chapters == STAGE['chapters'], chapters
    rows = script_rows(crate)
    printed = step('script', [binary, 'script']).decode()
    assert printed == ''.join('%s\t%s\n' % row for row in rows), 'binary script differs from script.tsv'
    owned = check_script(rows, chapters)
    ids = [key for key, _ in rows]
    modules = static_cues(crate, ids)

    ffmpeg = Path(P['ffmpeg'])
    cue_out = job/'cue-probe'
    step('cue-probe', [binary, 'render', 'all', '--scratch-voice', '--res', '64x36', '--fps', '1', '--out', cue_out],
         extra=dict(PATH=str(ffmpeg.parent) + ':/usr/bin:/bin'))
    said = said_cues(cue_out, chapters, owned, ids)
    glyph_out = job/'glyph-probe'
    glyph_text = step('glyphs', [binary, 'glyphs', glyph_out, '--res', '1920x1080']).decode()
    glyphs = check_glyphs(glyph_text, glyph_out, chapters)

    check_files(files)
    assert listing(source) == (STAGE['files'], STAGE['tree_sha256']), 'the source tree changed during the build'
    assert listing(crate) == crate_listing, 'the crate Cargo read changed during the build'
    with (job/'qualification.json').open('x') as out:
        json.dump(dict(schema='exponential-build716', host=P['name'], node=platform.node(), profile=P['profile'],
                       package=STAGE['package'], binary=str(binary), binary_sha256=sha(binary),
                       source_tree_sha256=STAGE['tree_sha256'], handoff_sha256=STAGE['handoff_sha256'],
                       crate=str(crate), crate_listing=crate_listing, job_copy_edits=edits,
                       inputs_sha256=sha(manifest), input_files=len(files), preflight_sha256=sha(job/'preflight716.json'),
                       previous_binary_sha256=previous, toolchain_sha256=tool_record['sha256'],
                       tests_passed=int(counts[0]), original_tests_passed=len(ORIGINAL_TESTS),
                       tests_sha256=hashlib.sha256('\n'.join(sorted(passed)).encode()).hexdigest(), clippy_passed=True, facts=len(native),
                       facts_sha256=hashlib.sha256(facts).hexdigest(), oracle_sha256=STAGE['oracle_sha256'],
                       chapters=chapters, cues=ids, cues_per_chapter={k: len(v) for k, v in owned.items()},
                       cues_said_in_script_order=said == ids, chapter_modules=modules,
                       ffmpeg=dict(path=str(ffmpeg), sha256=sha(ffmpeg)), glyphs=glyphs, seconds=TIMES,
                       scope='Native automated checks of expanded-stage12 only: tests, Clippy, facts against the Python '
                             'oracle, chapter and cue coverage, and a glyph probe with zero failures whose pages no '
                             'one has read. Cue timing is estimated. No narration was generated or checked. No '
                             'timeline, frame, picture, motion, audio, fine-sync, AV or release acceptance.'), out, indent=2)
    print(json.dumps(dict(passed=True, job=job.name, host=P['name'], binary_sha256=sha(binary))))
    return 0


def main():
    p = argparse.ArgumentParser(description=__doc__.split('\n', 1)[0])
    p.add_argument('mode', choices=('inputs', 'run'))
    p.add_argument('--host', required=True, choices=('csd', 'mac'))
    p.add_argument('--job', required=True)
    p.add_argument('--source', required=True)
    args = p.parse_args()
    assert re.fullmatch(r'[a-z0-9-]+', args.job), 'job must be a simple lowercase name'
    check_pins()
    P = host_profile(args.host)
    check_host(P)
    return prepare_inputs(args, P) if args.mode == 'inputs' else run(args, P)


if __name__ == '__main__':
    raise SystemExit(main())
