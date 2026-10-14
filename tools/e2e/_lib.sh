#!/bin/sh
# Shared NDJSON logging helper for tools/e2e/ scripts (plan §15.3).
# Never deletes files (AGENTS.md Rule #1).
set -eu

E2E_ROOT=$(cd "$(dirname "$0")/../.." && pwd)
E2E_SCRIPT=$(basename "$0" .sh)
E2E_STAMP=$(date -u +%Y%m%dT%H%M%SZ)
mkdir -p "$E2E_ROOT/logs/e2e"
E2E_LOG="$E2E_ROOT/logs/e2e/${E2E_SCRIPT}-${E2E_STAMP}.ndjson"
: > "$E2E_LOG"
E2E_STEPS=0
E2E_PASSED=0

# e2e_step STEP_NAME CMD [ARGS...]
# Runs CMD [ARGS...], records elapsed seconds, exit status, and optional
# E2E_METRICS_JSON / E2E_ARTIFACTS (space-separated paths) into $E2E_LOG.
e2e_step() {
  step_name=$1
  shift
  E2E_STEPS=$((E2E_STEPS + 1))
  cmd_str="$*"
  t0=$(python3 -I -c 'import time; print(f"{time.time():.6f}")')
  rc=0
  "$@" || rc=$?
  t1=$(python3 -I -c 'import time; print(f"{time.time():.6f}")')
  python3 -I - "$E2E_LOG" "$step_name" "$cmd_str" "$rc" "$t0" "$t1" "${E2E_METRICS_JSON:-{}}" "${E2E_ARTIFACTS:-}" <<'PY'
import hashlib
import json
import pathlib
import sys

log_path = pathlib.Path(sys.argv[1])
step = sys.argv[2]
cmd = sys.argv[3]
rc = int(sys.argv[4])
t0 = float(sys.argv[5])
t1 = float(sys.argv[6])
try:
    metrics = json.loads(sys.argv[7]) if sys.argv[7] else {}
except Exception:
    metrics = {"raw": sys.argv[7]}
artifacts = []
for raw in sys.argv[8].split():
    p = pathlib.Path(raw)
    if p.is_file():
        data = p.read_bytes()
        artifacts.append({
            "path": str(p),
            "bytes": len(data),
            "sha256": hashlib.sha256(data).hexdigest(),
        })
record = {
    "step": step,
    "cmd": cmd,
    "exit": rc,
    "seconds": round(max(0.0, t1 - t0), 4),
    "metrics": metrics,
    "artifacts": artifacts,
}
with log_path.open("a", encoding="utf-8") as f:
    f.write(json.dumps(record, sort_keys=True) + "\n")
print(f"[{ 'PASS' if rc == 0 else 'FAIL' }] {step} ({record['seconds']:.2f}s)")
PY
  E2E_METRICS_JSON="{}"
  E2E_ARTIFACTS=""
  if [ "$rc" -ne 0 ]; then
    echo "e2e: $E2E_SCRIPT FAILED at step '$step_name' (exit $rc); log: $E2E_LOG" >&2
    exit "$rc"
  fi
  E2E_PASSED=$((E2E_PASSED + 1))
}

e2e_summary() {
  echo "e2e: $E2E_SCRIPT PASS ($E2E_PASSED/$E2E_STEPS steps) -> $E2E_LOG"
}
