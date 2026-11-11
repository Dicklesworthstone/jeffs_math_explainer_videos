#!/bin/sh
# Root-owned serial lane for pictures1008, for use ONLY after verify1005 is terminal, stable and pinned
# (pin1008.py check, then pin1008.py write to $Q/pictures1008.py), and only after root and CC2 have reviewed 1008.
# Not run by CC1. usage: sh pictures1008-serial01.sh [KEY ...]   (no keys = all 15 in qualification.json order)
# Stops at the first refusal or failure. A refused or failed chapter keeps its job directory and its name cannot be
# reused, so do not rerun a key that already has a job directory; pass only the remaining keys.
# Do not mix with 1007: 1008 refuses any chapter that has a pictures1005/1006/1007 job.
set -eu
D=/data/projects/jmev-css-render-worker-20261107
Q=/data/projects/jmev-css-render-worker-20261107/qualification01/exponential933
H=/data/projects/jmev-css-render-worker-20261107/qualification01/exponential933/pictures1008.py
if [ "$#" -eq 0 ]; then
  set -- 01_rate_law 02_factorials 03_product_grid 04_tiny_multipliers 05_whole_sum 06_only_solution \
    07_calculating_ruler 08_decay_law 09_charging_gap 10_changing_rates 11_money_limits 12_chance_and_limits \
    13_turning_rate 14_euler_identity 15_logarithm_bridge
fi
/usr/bin/sha256sum "$H"
for K in "$@"; do
  J=css-exponential-master-pictures1008-$K
  test ! -e "$D/jobs/$J"
  test ! -e "$Q/$J-inputs.json"
  echo "$(date -u +%FT%TZ) make-inputs $K"
  /usr/bin/python3 -I -B "$H" make-inputs --phase pictures --chapter "$K"
  echo "$(date -u +%FT%TZ) run $K"
  /usr/bin/python3 -I -B "$D/guard_css74.py" --phase native --job "$J" --inputs "$Q/$J-inputs.json" -- \
    /usr/bin/python3 -I -B "$H" run --phase pictures --chapter "$K"
  echo "$(date -u +%FT%TZ) done $K"
done
echo "$(date -u +%FT%TZ) all requested chapters exited 0; run the recheck next:"
echo "  /usr/bin/python3 -I -B $H make-inputs --phase recheck"
echo "  /usr/bin/python3 -I -B $D/guard_css74.py --phase native --job css-exponential-pictures1008-recheck --inputs $Q/css-exponential-pictures1008-recheck-inputs.json -- /usr/bin/python3 -I -B $H run --phase recheck"
