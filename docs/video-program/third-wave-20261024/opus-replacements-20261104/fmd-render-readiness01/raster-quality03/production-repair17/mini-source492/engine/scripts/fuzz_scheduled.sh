#!/usr/bin/env bash
# fm-ntp (ADR-0003 class=fuzz): the SCHEDULED coverage-guided fuzz job for
# the fmn-codec untrusted-input parsers. This is NEVER a merge gate —
# scripts/check.sh does not call it. CI invokes it from the weekly
# `fuzz-codec` job in .github/workflows/ci.yml (`schedule` events only):
#
#     cargo install cargo-fuzz --version 0.13.1
#     bash scripts/fuzz_scheduled.sh
#
# (Not --locked: cargo-fuzz 0.13.1's committed lockfile pins a rustix
# the pinned nightly refuses to compile; see the ci.yml install step.)
#
# Each target runs time-boxed. A crash (assertion failure = a budget
# violation, libFuzzer-detected panic, timeout-hang, or rss blow-out)
# fails this script loudly and drops the reproducer under fuzz/artifacts/:
# that is a FINDING — a real decoder bug to fix at the source in
# fmn-codec, never a harness to weaken.
#
# Environment knobs:
#   FMN_FUZZ_SECONDS    per-target wall-clock budget (default 60)
#   FMN_FUZZ_TIMEOUT    per-input hang timeout seconds (default 25)
#   FMN_FUZZ_RSS_MB     per-input rss cap in MiB (default 2048)
#   FMN_FUZZ_TARGET_DIR private build dir for the fallback (default fuzz/target)
set -euo pipefail
cd "$(dirname "$0")/.."

# Not SECONDS: bash counts that variable up from any value assigned to it,
# so every later "(Ns)" progress line reported a growing budget.
BUDGET_SECONDS="${FMN_FUZZ_SECONDS:-60}"
TIMEOUT="${FMN_FUZZ_TIMEOUT:-25}"
RSS_MB="${FMN_FUZZ_RSS_MB:-2048}"
FALLBACK_TARGET_DIR="${FMN_FUZZ_TARGET_DIR:-fuzz/target}"
TARGETS=(inflate_bytes decode_png decode_jpeg decode_entry_envelope namespace_and_object_paths svg_document yaml_config tex_math ttf_font_parse path_boolean)

mkdir -p fuzz/artifacts

run_flags=(
    "-max_total_time=${BUDGET_SECONDS}"
    "-timeout=${TIMEOUT}"
    "-rss_limit_mb=${RSS_MB}"
    "-artifact_prefix=fuzz/artifacts/"
    "-print_final_stats=1"
)

status=0

if cargo fuzz --version >/dev/null 2>&1; then
    # Primary path: cargo-fuzz builds with sanitizer-coverage
    # instrumentation (the coverage-guided mode this campaign is for).
    for target in "${TARGETS[@]}"; do
        echo "==> cargo fuzz run ${target} (${BUDGET_SECONDS}s)"
        if ! cargo fuzz run "${target}" -- "${run_flags[@]}"; then
            echo "==> FINDING: ${target} crashed — reproducer in fuzz/artifacts/" >&2
            status=1
        fi
    done
else
    # Fallback for machines without the cargo-fuzz subcommand: build the
    # harness binaries directly into a private target dir and drive
    # libFuzzer by hand. This mode lacks coverage-instrumentation flags,
    # so guidance is weaker, but the budget assertions, hang timeout, rss
    # cap, and corpus replay all still hold.
    echo "==> cargo-fuzz not found; building harnesses directly (degraded guidance)"
    cargo build --release --manifest-path fuzz/Cargo.toml \
        --target-dir "${FALLBACK_TARGET_DIR}"
    for target in "${TARGETS[@]}"; do
        echo "==> ${target} (${BUDGET_SECONDS}s)"
        if ! "${FALLBACK_TARGET_DIR}/release/${target}" \
            "fuzz/corpus/${target}" "${run_flags[@]}"; then
            echo "==> FINDING: ${target} crashed — reproducer in fuzz/artifacts/" >&2
            status=1
        fi
    done
fi

if [[ "${status}" -ne 0 ]]; then
    echo "==> fuzz campaign found crashes; see fuzz/artifacts/ (fix at the source in fmn-codec)" >&2
    exit "${status}"
fi
echo "==> all ${#TARGETS[@]} targets clean within budget"
