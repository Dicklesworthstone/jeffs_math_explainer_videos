#!/usr/bin/env python3
"""Run the binder's build() with its pins replaced by the hashes of the tree in the cwd.

    python3 -I -B <this file> <binder file>       from a mirror tree, never the repo

For planted-defect tests only: it lets a planted change reach the binder's semantic stops, which
the stale-input refusal and the script's own hash check would otherwise hide. It prints "BUILT" or
"STOP: <message>" and writes nothing. Not a way to bind anything.
"""
import importlib.util
import sys
from pathlib import Path

sys.dont_write_bytecode = True


def main():
    if not Path("mirror-tree.marker").is_file():
        sys.exit("refused: this runs only in a mirror tree")
    spec = importlib.util.spec_from_file_location("binder_under_test", sys.argv[1])
    binder = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(binder)
    binder.CH_LISTING = tuple(sorted(p.name for p in binder.CH.iterdir()))
    binder.PINS = {p: binder.file_sha(p) for p in binder.pin_paths() if Path(p).is_file()}
    binder.EXPECTED_SCRIPT = binder.file_sha(binder.STAGE / "script.tsv")
    try:
        binder.refuse_stale_inputs()
        binder.build()
    except SystemExit as stop:
        print(f"STOP: {stop}")
        return 0
    print("BUILT")
    return 0


if __name__ == "__main__":
    sys.exit(main())
