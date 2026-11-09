#!/usr/bin/env bash
# The mandatory local/owned-host verification gate (AGENTS.md): fmt, check,
# clippy -D warnings, rustdoc -D warnings, test, then the structural crate-DAG
# check — in order, stopping on first failure. Hosted workflows may invoke this
# script, but hosted CI availability is not part of the correctness contract.
set -euo pipefail
cd "$(dirname "$0")/.."

# Tests whose inputs are absent (the pinned Reference checkout, the private TeX
# corpus, a built wasm artifact) record a SKIPPED line here instead of passing
# silently; set FMN_REQUIRE_FULL_INPUTS=1 to make any such skip fail (fm-5wq.48).
FMN_SKIP_LEDGER="${FMN_SKIP_LEDGER:-$(mktemp)}"
export FMN_SKIP_LEDGER

echo "==> agent control-plane parser/planner/generator/tests"
python3 -m py_compile \
    scripts/agent_brief.py \
    scripts/agent_next.py \
    scripts/agent_task_semantics.py \
    scripts/agent_claim_policy.py \
    scripts/agent_claim_guard.py \
    scripts/agent_claim.py \
    scripts/generate_agent_brief.py \
    scripts/test_agent_brief.py \
    scripts/test_agent_brief_strict_json.py \
    scripts/test_agent_next.py \
    scripts/test_agent_next_output.py \
    scripts/test_agent_scope.py \
    scripts/test_agent_claim_policy.py \
    scripts/test_agent_claim_guard.py \
    scripts/test_agent_task_semantics.py \
    scripts/test_agent_claim.py \
    scripts/test_agent_claim_subprocess.py \
    scripts/test_generate_agent_brief.py \
    scripts/test_generate_agent_brief_io.py
# Invalid graph or activation state must fail before any machine payload.
python3 scripts/agent_brief.py --format json --limit 1 --check >/dev/null
python3 scripts/test_agent_brief.py
python3 scripts/test_agent_brief_strict_json.py
python3 scripts/test_agent_next.py
python3 scripts/test_agent_next_output.py
python3 scripts/test_agent_scope.py
python3 scripts/test_agent_claim_policy.py
python3 scripts/test_agent_claim_guard.py
python3 scripts/test_agent_task_semantics.py
python3 scripts/test_agent_claim.py
python3 scripts/test_agent_claim_subprocess.py
python3 scripts/test_generate_agent_brief.py
python3 scripts/test_generate_agent_brief_io.py
# Render the complete live ledger through every operational projection without
# mutating documentation. The claim planner proves that non-epic parents with
# live children cannot be returned as autonomous leaf work. Exact claim-policy
# labels additionally keep manual or externally gated leaves visible without
# allowing them to become autonomous recommendations. The claim guard then
# revalidates the exact graph, policy, schema contract, and recommendation.
# When work is claimable, the executor additionally acquires its repository-
# local lock and renders the exact atomic `br update --claim` argv plus bounded
# timeout/output policy in dry-run mode; it never invokes br or mutates Beads.
python3 scripts/agent_next.py --format json --check >/dev/null
claim_token="$(python3 scripts/agent_claim_guard.py --format token)"
python3 scripts/agent_claim_guard.py \
    --expect-token "$claim_token" \
    --format json >/dev/null
claim_id="${claim_token##*:}"
if [[ "$claim_id" != "none" ]]; then
    python3 scripts/agent_claim.py \
        --expect-token "$claim_token" \
        --issue "$claim_id" \
        --assignee fmn-check-gate \
        --command-timeout-seconds 60 \
        --command-output-budget-bytes 16777216 \
        --dry-run >/dev/null
fi
python3 scripts/generate_agent_brief.py --stdout >/dev/null

echo "==> Python portal refusal inventory"
python3 -m py_compile \
    scripts/audit_portal_refusals.py \
    scripts/test_audit_portal_refusals.py
python3 scripts/test_audit_portal_refusals.py
python3 scripts/audit_portal_refusals.py --check >/dev/null

echo "==> corpus speed-ratio producer statistics"
python3 -m py_compile \
    scripts/corpus_speed_ratio.py \
    scripts/test_corpus_speed_ratio.py
python3 scripts/test_corpus_speed_ratio.py

echo "==> Python portal runtime-parity audit contract"
python3 -m py_compile \
    crates/fmn-python/python/fmn_python/schema_provenance.py \
    crates/fmn-python/python/fmn_python/parity_audit.py \
    crates/fmn-python/python/fmn_python/__main__.py \
    crates/fmn-python/python/manimlib/__init__.py \
    crates/fmn-python/python/manimlib/_animation_semantics.py \
    scripts/test_animation_semantics_installer.py \
    scripts/test_python_transform_paths.py \
    scripts/audit_portal_runtime.py \
    scripts/verify_portal_runtime_receipt.py \
    scripts/test_schema_provenance.py \
    scripts/test_audit_portal_runtime.py \
    scripts/test_portal_parity_cli.py \
    scripts/test_verify_portal_runtime_receipt.py
bash -n scripts/check_portal_runtime.sh
python3 scripts/test_schema_provenance.py
python3 scripts/test_animation_semantics_installer.py
python3 scripts/test_python_transform_paths.py
python3 scripts/test_audit_portal_runtime.py
python3 scripts/test_portal_parity_cli.py
python3 scripts/test_verify_portal_runtime_receipt.py

echo "==> Python API extractor and geometry helper authority policy"
python3 -m py_compile \
    scripts/gen_api_schema.py \
    scripts/test_gen_api_schema.py \
    crates/fmn-python/python/fmn_python/library_constructor_authority.py \
    scripts/check_python_helper_aliases.py \
    scripts/test_python_helper_aliases.py \
    scripts/check_library_constructor_authority.py \
    scripts/test_library_constructor_authority.py
python3 scripts/test_gen_api_schema.py
python3 scripts/check_python_helper_aliases.py
python3 scripts/test_python_helper_aliases.py
python3 scripts/check_library_constructor_authority.py
python3 scripts/test_library_constructor_authority.py

echo "==> native installer smoke"
bash scripts/test_install.sh

echo "==> cargo fmt --check"
cargo fmt --check

echo "==> cargo check --all-targets"
cargo check --all-targets

echo "==> shipped fmn refuses a cli-only feature selection"
if cli_only_output=$(cargo check -p fmn-cli --no-default-features --features cli --bin fmn 2>&1); then
    echo "ERROR: the shipped fmn binary built without the batch product axis" >&2
    exit 1
fi
if [[ "$cli_only_output" != *"requires the features:"* \
    || "$cli_only_output" != *"batch"* \
    || "$cli_only_output" != *"cli"* ]]; then
    printf 'ERROR: cli-only negative control failed for an unexpected reason:\n%s\n' \
        "$cli_only_output" >&2
    exit 1
fi

echo "==> cargo check -p fmn-cli --features batch --all-targets"
cargo check -p fmn-cli --features batch --all-targets

echo "==> fmn CLI smoke (default-feature parser floor)"
cargo test -p fmn-cli --test cli_smoke

echo "==> fmn CLI smoke (complete shipping binary)"
cargo test -p fmn-cli --features batch --test cli_smoke

echo "==> fmn doctor smoke (default-feature report contract)"
cargo test -p fmn-cli --test doctor_smoke

echo "==> fmn doctor smoke (complete shipping binary)"
cargo test -p fmn-cli --features batch --test doctor_smoke

echo "==> cargo check -p fmn-output --features ffmpeg-test-fixture --all-targets"
cargo check -p fmn-output --features ffmpeg-test-fixture --all-targets

# Cargo accepts a misspelled filter with zero tests and exit 0. Require all
# real-extension acceptance suites in the compiled inventory before running.
echo "==> native Python production acceptance"
native_acceptance_tests="$(cargo test -p fmn-python --lib -- --list 2>&1)"
for suite in production_bridge_acceptance_suite production_animation_semantics_acceptance_suite production_native_output_acceptance_suite; do
    if [[ $'\n'"$native_acceptance_tests"$'\n' != *$'\n'"tests::$suite: test"$'\n'* ]]; then
        printf 'ERROR: native Python acceptance binary is missing %s\n' "$suite" >&2
        exit 1
    fi
done
cargo test -p fmn-python --lib production_

# W5 wasm tier 1 (fm-l97, §10.7): the recorded wasm32 gate. The named crates
# are the render axis the browser build compiles; this must stay green for
# the fmn-wasm surface to have a foundation. Requires the
# wasm32-unknown-unknown rustup target.
echo "==> cargo check --target wasm32-unknown-unknown (W5 wasm tier-1 axis)"
cargo check --target wasm32-unknown-unknown -p fmn-render -p fmn-scene -p fmn-geom -p fmn-mobject -p fmn-anim -p fmn-core

echo "==> cargo clippy --all-targets -- -D warnings"
cargo clippy --all-targets -- -D warnings

echo "==> cargo clippy -p fmn-cli --features batch --all-targets -- -D warnings"
cargo clippy -p fmn-cli --features batch --all-targets -- -D warnings

echo "==> cargo clippy -p fmn-output --features ffmpeg-test-fixture --all-targets -- -D warnings"
cargo clippy -p fmn-output --features ffmpeg-test-fixture --all-targets -- -D warnings

echo "==> cargo doc --no-deps --workspace (rustdoc -D warnings)"
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace

if [[ "${FMN_SIMD_TIER_GATE:-0}" == "1" ]]; then
    # The weekly x86-64-v4 artifact runs under user-mode emulation. Compile and
    # lint the whole workspace above, then execute the crates that own SIMD
    # kernels plus the certified tier-equivalence corpus. Unrelated subprocess
    # suites would escape the Cargo target runner and execute on the host ISA.
    echo "==> cargo test (SIMD-owning crates)"
    cargo test -p fmn-render -p fmn-anim
    echo "==> cargo test (certified tier corpus)"
    cargo test -p fmn-conformance --test certified_engine
else
    echo "==> cargo test"
    cargo test
    echo "==> cargo test -p fmn-cli --features batch --all-targets"
    cargo test -p fmn-cli --features batch --all-targets
    echo "==> cargo test -p fmn-output --features ffmpeg-test-fixture --all-targets"
    cargo test -p fmn-output --features ffmpeg-test-fixture --all-targets
fi

echo "==> crate-DAG check (workspace graph vs plan §19)"
python3 scripts/check_crate_dag.py

# W5 wasm tier 1 headless smoke: instantiate the compiled probe in a real JS
# wasm VM and prove the render path executes there deterministically, the
# browser clock capability reads the host clocks, the process capability
# fails closed, and the topology is single-CPU. Skipped (loudly) only where
# no JS runtime exists; owned release hosts carry node.
if command -v node >/dev/null 2>&1; then
    echo "==> wasm32 headless smoke (node)"
    ./wasm-smoke/run.sh
else
    echo "==> wasm32 headless smoke SKIPPED: no node on PATH (wasm-smoke/run.sh needs node or bun)"
fi

# W11 npm/WASM packaging is a release gate because it deliberately requires
# wasm-pack, npm, webpack, and Chromium in addition to the governed Rust
# closure. Opt in on release hosts; the script itself fails closed on tool
# versions, artifact freshness, package inventory, size, and browser behavior.
if [[ "${FMN_WASM_PACKAGE_GATE:-0}" == "1" ]]; then
    echo "==> npm/WASM package + real-browser release gate"
    scripts/check_wasm_package.sh
else
    echo "==> npm/WASM package release gate SKIPPED: set FMN_WASM_PACKAGE_GATE=1"
fi

if [[ -s "$FMN_SKIP_LEDGER" ]]; then
    echo "==> skipped for missing inputs (set FMN_REQUIRE_FULL_INPUTS=1 to fail instead):"
    sort -u "$FMN_SKIP_LEDGER"
fi
echo "OK: all gates green"
