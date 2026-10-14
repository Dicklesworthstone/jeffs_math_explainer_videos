#!/bin/sh
# End-to-end driver & gate-transition harness (plan §3.6, bead mx-e-w1-4qa.11):
# Runs tools/test_driver.py and drives status/gate transitions on a scaffolded
# throwaway explainer (local only; nothing in the checkout is modified or deleted).
. "$(dirname "$0")/_lib.sh"

e2e_step "unit_test_driver" python3 -I "$E2E_ROOT/tools/test_driver.py"

WORK=$(python3 -I -c 'import tempfile; print(tempfile.mkdtemp(prefix="e2e_driver_"))')
mkdir -p "$WORK/explainer_corpus"
cp "$E2E_ROOT/x" "$WORK/x"
chmod +x "$WORK/x"
cp "$E2E_ROOT/Cargo.toml" "$WORK/Cargo.toml"
ln -s "$E2E_ROOT/templates" "$WORK/templates"
cp -R "$E2E_ROOT/tools" "$WORK/tools"

e2e_step "scaffold_throwaway" "$WORK/x" new throwaway_demo "Throwaway Demo"
e2e_step "storyboard_gate" "$WORK/x" throwaway_demo storyboard
e2e_step "reviewed_draft_gate" "$WORK/x" throwaway_demo reviewed draft
e2e_step "status_report" "$WORK/x" throwaway_demo status
e2e_step "doctor_report" "$WORK/x" doctor
e2e_summary
