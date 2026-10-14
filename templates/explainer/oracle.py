#!/usr/bin/env python3
"""Independent oracle for every number in the video (docs/fact-sheet.md).

    ./x __SLUG__ facts-check
    uv run --no-project --with numpy python3 -I oracle.py [--json]

Prints `key<TAB>value` lines with the SAME keys `__SLUG__ facts` prints;
`./x __SLUG__ facts-check` diffs the two. With `--json`, prints a JSON object
mapping each key to its value. Compute each value a different way from the Rust
code (another library, the definition, a published table), never by porting the
Rust.
"""

import json
import math
import sys


def facts():
    n = 5000
    k = 4
    total = math.comb(n, k)
    yield "points", n
    yield "pick", k
    yield "quadruples_5000", total
    yield "quadruples_trillions_floor", total // 10**12


def main(argv):
    rows = list(facts())
    if "--json" in argv:
        print(json.dumps({key: value for key, value in rows}, indent=2))
        return 0
    for key, value in rows:
        print(f"{key}\t{value}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
