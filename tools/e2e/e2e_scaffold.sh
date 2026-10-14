#!/bin/sh
# End-to-end scaffold check (plan §15.3, beads mx-e-w1-4qa.10 & mx-e-w1-4qa.12):
# Scaffolds `demo_explainer` in an isolated temporary workspace copy (symlinking
# crates/ and Cargo.lock so cargo reuses the existing target cache without
# mutating the working tree), then verifies:
#   1. `./x new demo_explainer "Demo"`
#   2. `./x demo_explainer storyboard`
#   3. `cargo test -p demo_explainer`
#   4. `./x demo_explainer facts-check`
#   5. `./x demo_explainer status --porcelain`
. "$(dirname "$0")/_lib.sh"

WORK=$(python3 -I -c 'import tempfile; print(tempfile.mkdtemp(prefix="e2e_scaffold_"))')
mkdir -p "$WORK/explainer_corpus"
cp "$E2E_ROOT/x" "$WORK/x"
chmod +x "$WORK/x"
cp "$E2E_ROOT/Cargo.toml" "$WORK/Cargo.toml"
[ -f "$E2E_ROOT/Cargo.lock" ] && cp "$E2E_ROOT/Cargo.lock" "$WORK/Cargo.lock"
[ -f "$E2E_ROOT/rust-toolchain.toml" ] && cp "$E2E_ROOT/rust-toolchain.toml" "$WORK/rust-toolchain.toml"
[ -f "$E2E_ROOT/project.env" ] && cp "$E2E_ROOT/project.env" "$WORK/project.env"
ln -s "$E2E_ROOT/crates" "$WORK/crates"
ln -s "$E2E_ROOT/templates" "$WORK/templates"
cp -R "$E2E_ROOT/tools" "$WORK/tools"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$E2E_ROOT/target}"

e2e_step "scaffold_new" "$WORK/x" new demo_explainer "Demo"
E2E_ARTIFACTS="$WORK/explainer_corpus/demo_explainer/Cargo.toml $WORK/explainer_corpus/demo_explainer/script.tsv"
e2e_step "storyboard_check" "$WORK/x" demo_explainer storyboard
e2e_step "cargo_test_demo" cargo test --manifest-path "$WORK/Cargo.toml" -p demo_explainer -Zgit=shallow-deps -Zgitoxide=fetch
e2e_step "facts_check_demo" "$WORK/x" demo_explainer facts-check
e2e_step "status_porcelain" "$WORK/x" demo_explainer status --porcelain
e2e_summary
