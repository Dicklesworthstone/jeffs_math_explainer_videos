#!/bin/bash
# Root runs this on css, in tmux, after the transfer. RUN is a new two-digit number each time, SEAL is
# the sha256 of this folder's SHA256SUMS, and FONT is the font's sha256 from `check`. A repeated RUN is
# refused before it can touch that run's log or exit file.
set -uo pipefail
case "${1:-}" in
    [0-9][0-9]) run="$1" ;;
    *) echo "usage: start-overlay-css02.sh RUN SEAL FONT_SHA256" >&2; exit 2 ;;
esac
seal="${2:?SEAL}"
font="${3:?FONT_SHA256}"
export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin"
HERE="$(cd "$(dirname "$0")" && pwd -P)" || exit 1
cd "$HERE/../../../../.." || exit 1
lane=overlay-lane02
log="$lane/overlay-controller$run.log"
mkdir -p "$lane" || exit 1
( set -C; : > "$log" ) 2>/dev/null || { echo "refused: $log already exists" >&2; exit 1; }
/usr/bin/python3 -I -B "$HERE/overlay_css02.py" run --run "$run" --seal "$seal" --font-sha256 "$font" >> "$log" 2>&1
result=$?
printf '%s\n' "$result" > "$lane/overlay-controller$run.exit"
exit "$result"
