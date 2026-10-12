#!/bin/sh
# cue_check.sh [RENDER_DIR]: the corpus's cue check (cue_sheets.py) for this
# video's v5 line ids. RENDER_DIR defaults to renders/narrated.
#
# cue_sheets.py reads a line's chapter off its id (NN_k), and the v5 ids
# (h1..o4) name theirs by first letter, so `./x hoeffding_d narrated` renders
# and then fails every line with "belongs to another chapter". This checks a
# view of RENDER_DIR, written beside it as RENDER_DIR_cue_view, in which every
# id carries the number of the chapter `hoeffding owners` gives it (h1 becomes
# 01_h1). A line spoken in the wrong chapter is therefore still an error. The
# sheets go where ./x narrated puts them: RENDER_DIR/cue_sheets.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
# shellcheck source=/dev/null
[ -f "$root/project.env" ] && . "$root/project.env"
SKILLS="${SKILLS:-$HOME/.claude/skills}"
check="$SKILLS/math-explainer-videos-end-to-end-with-franken-manim-frankentts/scripts/cue_sheets.py"
binary="${CARGO_TARGET_DIR:-$root/target}/release/hoeffding"
dir=${1:-$here/renders/narrated}
[ -d "$dir" ] || { echo "cue_check: no renders at $dir" >&2; exit 2; }
[ -x "$binary" ] || { echo "cue_check: no binary at $binary (./x hoeffding_d build)" >&2; exit 2; }
[ -f "$check" ] || { echo "cue_check: no cue_sheets.py under $SKILLS" >&2; exit 2; }
dir=$(cd "$dir" && pwd)
view="${dir}_cue_view"
mkdir -p "$view"

"$binary" owners > "$view/owners.tsv"
# id -> NN_id by its owner; an id with no owner stays as it is, and fails the check
# shellcheck disable=SC2016  # awk's fields, not the shell's
prefixed='NR == FNR { own[$1] = substr($2, 1, 2); next }
  /^#/ || !/\t/ { next }
  { if ($1 in own) $1 = own[$1] "_" $1; print }'
awk -F'\t' -v OFS='\t' "$prefixed" "$view/owners.tsv" "$here/script.tsv" > "$view/script.tsv"
keys=$("$binary" chapters)
for k in $keys; do
  [ -f "$dir/$k.mp4" ] && [ -f "$dir/$k.cues.tsv" ] || continue
  ln -sf "$dir/$k.mp4" "$view/$k.mp4"
  awk -F'\t' -v OFS='\t' "$prefixed" "$view/owners.tsv" "$dir/$k.cues.tsv" > "$view/$k.cues.tsv"
done
exec python3 -I "$check" "$view" --script "$view/script.tsv" \
  --chapters "$(printf '%s' "$keys" | paste -sd, -)" --sheets "$dir/cue_sheets"
