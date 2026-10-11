#!/bin/sh
# ./x: the corpus driver (plan §3.6; bead W1 .11 grows it into the full
# corpus-aware port of the template's driver). Commands so far:
#
#   ./x <slug> review storyboard|script|draft   persona review packet (plan §5.2)
set -eu
here=$(cd "$(dirname "$0")" && pwd)
usage() { sed -n '2,5p' "$0" | sed 's/^# \{0,1\}//'; exit 2; }
[ $# -ge 2 ] || usage
slug=$1; cmd=$2; shift 2
case "$slug" in
  */*|.*|'') echo "x: bad slug '$slug'" >&2; exit 2 ;;
esac
case "$cmd" in
  review)
    [ $# -eq 1 ] || usage
    exec python3 -I "$here/tools/review.py" "$slug" "$1" ;;
  *) usage ;;
esac
