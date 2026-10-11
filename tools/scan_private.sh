#!/bin/sh
# tools/scan_private.sh: fail when a file names a private machine, account or
# path. This repository is public (AGENTS.md, "Public-Repository Hygiene").
#
#   tools/scan_private.sh            every tracked file except .beads/
#   tools/scan_private.sh FILE...    just those files (tracked or not)
#
# Built in: absolute home paths, agent scratch paths, Tailscale (CGNAT
# 100.64.0.0/10) addresses. Machine and account names are private too, so
# they are not written here: list them, one extended regex per line, in
# .private_patterns (gitignored), or pass them newline-separated in
# PRIVATE_PATTERNS (CI reads a repository secret). .beads/ is skipped: br
# records its own metadata there (the repo path, the OS user).
# Exit 0 clean, 1 on any finding (printed as file:line:text).
set -eu
cd "$(dirname "$0")/.."

generic='/(Users|home)/[A-Za-z][A-Za-z0-9_.-]*/|/private/tmp/|/tmp/claude-|(^|[^0-9.])100\.(6[4-9]|[7-9][0-9]|1[01][0-9]|12[0-7])\.[0-9]{1,3}\.[0-9]{1,3}([^0-9]|$)'

patterns=$(mktemp)
trap 'rm -f "$patterns"' EXIT
printf '%s\n' "$generic" > "$patterns"
[ -f .private_patterns ] && grep -v '^[[:space:]]*\(#\|$\)' .private_patterns >> "$patterns"
[ -n "${PRIVATE_PATTERNS:-}" ] && printf '%s\n' "$PRIVATE_PATTERNS" | grep -v '^[[:space:]]*$' >> "$patterns"

if [ $# -gt 0 ]; then
  found=$(grep -nIiE -f "$patterns" -- "$@" || true)
else
  # this file names the generic patterns; nothing else is exempt
  found=$(git grep -nIiE -f "$patterns" -- . ':!.beads' ':!tools/scan_private.sh' || true)
fi
if [ -n "$found" ]; then
  printf '%s\n' "$found" | cut -c1-200
  echo "scan_private: $(printf '%s\n' "$found" | wc -l | tr -d ' ') private reference(s); this repository is public" >&2
  exit 1
fi
echo "scan_private: clean ($(wc -l < "$patterns" | tr -d ' ') patterns)"
