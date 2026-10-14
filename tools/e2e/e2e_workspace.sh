#!/bin/sh
# End-to-end workspace & tools verification (plan §15.3, bead mx-e-w1-4qa.12):
# Verifies shared crates (explainer_score, explainer_kit), tool unit tests,
# script lints across the corpus, and scan_private.sh (including planted negative).
. "$(dirname "$0")/_lib.sh"

cd "$E2E_ROOT"
e2e_step "test_explainer_score" cargo test -p explainer_score --lib
e2e_step "test_explainer_kit" cargo test -p explainer_kit --lib -Zgit=shallow-deps -Zgitoxide=fetch
e2e_step "test_driver_py" python3 -I "$E2E_ROOT/tools/test_driver.py"
e2e_step "test_prosody_qa_py" uv run --quiet --no-project --with numpy --with praat-parselmouth python3 -I "$E2E_ROOT/tools/test_prosody_qa.py"
e2e_step "scan_private_clean" "$E2E_ROOT/tools/scan_private.sh"

check_planted_negative() {
  tmp=$(python3 -I -c 'import tempfile; print(tempfile.mkstemp(prefix="scan_neg_")[1])')
  printf 'leaked path: /Users/%s/secret\n' "alice" > "$tmp"
  if "$E2E_ROOT/tools/scan_private.sh" "$tmp" >/dev/null 2>&1; then
    return 1
  fi
  return 0
}
e2e_step "scan_private_planted_negative" check_planted_negative
e2e_summary
