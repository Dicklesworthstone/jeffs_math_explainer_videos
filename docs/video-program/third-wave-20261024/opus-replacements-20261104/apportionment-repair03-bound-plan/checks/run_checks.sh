#!/bin/sh
# Reproduce every check on this bundle. Light: Python only, no cargo, render or audio.
#     sh docs/video-program/third-wave-20261024/opus-replacements-20261104/apportionment-repair03-bound-plan/checks/run_checks.sh EMPTY_SCRATCH
# Run from the repo root. Everything that writes works on copies inside EMPTY_SCRATCH.
set -eu
X=$1
B=docs/video-program/third-wave-20261024/opus-replacements-20261104/apportionment-repair03-bound-plan
L=narration_lab/third-wave-20261024/root/elevenlabs-apportionment-rebuild-20261104
S=$L/combined-repair-stage03/explainer_corpus/apportionment
C=explainer_corpus/apportionment
P=narration_lab/third-wave-20261024/root/five-film-release-20261027/apportionment-opus-rebuild-preview02/render
T=$B/explainer_corpus/apportionment

tree_digest() { find "$1" -type f | LC_ALL=C sort | while read -r f; do shasum -a 256 "$f"; done | shasum -a 256 | cut -c1-64; }

echo "== inputs before"
BEFORE_S=$(tree_digest "$S"); BEFORE_C=$(tree_digest "$C")
echo "staged03 topic tree digest $BEFORE_S"
echo "canonical topic tree digest $BEFORE_C"
shasum -a 256 tools/coherent_narration.py tools/storyboard.py "$L/checks01/bind_oliver_plan.py" "$L/checks01/validate_oliver_plan.py" "$L/checks01/phrase_scan.py"

echo; echo "== 1. binder: outputs on disk equal what the staged source gives"
python3 -I -B "$B/checks/bind_plan.py" "$S" "$C" "$P" "$B" --check

echo; echo "== 2. storyboard corrections: output on disk equals the corrections applied to canonical"
python3 -I -B "$B/checks/storyboard_corrections.py" "$C/docs/storyboard.md" "$T/docs/storyboard.md"

echo; echo "== 3. coherent API in an isolated root, map against source, planted defects"
mkdir -p "$X/validate"
python3 -I -B "$B/checks/validate_plan.py" "$S" "$C" "$B" "$X/validate"

echo; echo "== 4. root's validate_oliver_plan.py on a copy; only its map file name is changed, oliver03 to oliver04"
mkdir -p "$X/cand-a" "$X/validate-root"
cp -R "$S/." "$X/cand-a/"
cp -f "$T/narration_units.json" "$X/cand-a/narration_units.json"
cp -f "$T/docs/narration-direction-map-oliver04.json" "$X/cand-a/docs/narration-direction-map-oliver04.json"
sed 's/oliver03/oliver04/g' "$L/checks01/validate_oliver_plan.py" > "$X/validate_oliver_plan.oliver04.py"
python3 -I -B "$X/validate_oliver_plan.oliver04.py" . "$X/cand-a" "$X/validate-root"

echo; echo "== 5. root's bind_oliver_plan.py on a copy of staged03, compared row by row"
mkdir -p "$X/cand-b"
cp -R "$S/." "$X/cand-b/"
python3 -I -B "$L/checks01/bind_oliver_plan.py" "$X/cand-b"
python3 -I -B "$B/checks/compare_root_binder.py" "$X/cand-b" "$B"

echo; echo "== 6. root's phrase_scan.py on the registered chapters of staged03"
mkdir -p "$X/cand-c/src/chapters"
cp -f "$S/script.tsv" "$X/cand-c/script.tsv"
for f in one_more_seat shares largest_tails sixteenth dials not_every_time yardstick signposts cost rounding_rules webster_quota both_promises population no_rule recap draw thumbnail mod; do
  cp -f "$S/src/chapters/$f.rs" "$X/cand-c/src/chapters/$f.rs"
done
python3 -I -B "$L/checks01/phrase_scan.py" "$X/cand-c"

echo; echo "== 7. tools/storyboard.py --check in isolated roots: canonical baseline, corrected, planted word change"
for case in baseline corrected planted; do
  R=$X/sb-$case
  mkdir -p "$R/tools" "$R/explainer_corpus/apportionment/docs"
  cp -f tools/storyboard.py "$R/tools/storyboard.py"
  cp -f "$C/script.tsv" "$R/explainer_corpus/apportionment/script.tsv"
done
cp -f "$C/docs/storyboard.md" "$X/sb-baseline/explainer_corpus/apportionment/docs/storyboard.md"
cp -f "$T/docs/storyboard.md" "$X/sb-corrected/explainer_corpus/apportionment/docs/storyboard.md"
sed 's/^| 04_2 | The whole parts are still one/| 04_2 | The whole parts remain one/' "$T/docs/storyboard.md" > "$X/sb-planted/explainer_corpus/apportionment/docs/storyboard.md"
python3 -I -B "$X/sb-baseline/tools/storyboard.py" --check apportionment
python3 -I -B "$X/sb-corrected/tools/storyboard.py" --check apportionment
if python3 -I -B "$X/sb-planted/tools/storyboard.py" --check apportionment; then
  echo "NOT REFUSED: a changed word passed"; exit 1
else
  echo "planted word change refused, as it should be"
fi
cmp "$C/script.tsv" "$X/sb-corrected/explainer_corpus/apportionment/script.tsv"
echo "script.tsv unchanged by the storyboard check"

echo; echo "== inputs after"
AFTER_S=$(tree_digest "$S"); AFTER_C=$(tree_digest "$C")
echo "staged03 topic tree digest $AFTER_S"
echo "canonical topic tree digest $AFTER_C"
[ "$BEFORE_S" = "$AFTER_S" ] && echo "staged03 unchanged during this run" || echo "STAGED03 CHANGED during this run"
[ "$BEFORE_C" = "$AFTER_C" ] && echo "canonical topic unchanged during this run" || echo "CANONICAL TOPIC CHANGED during this run"
echo; echo "ALL CHECKS RAN"
