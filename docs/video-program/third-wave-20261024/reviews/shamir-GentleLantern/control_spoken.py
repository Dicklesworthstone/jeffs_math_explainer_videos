#!/usr/bin/env python3
"""Negative controls for check_shamir.py's source checks: each must report FAIL.

    python3 -I -B docs/video-program/third-wave-20261024/reviews/shamir-GentleLantern/control_spoken.py

Each control feeds source_checks a snapshot file with one deliberate edit
(made in memory; nothing on disk changes): an uncovered number word, a
SPOKEN phrase the line no longer says, and a production line that drifts
from the draft. Exit 0 when all three are caught.
"""

import importlib.util
import pathlib
import sys

HERE = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("check_shamir", HERE / "check_shamir.py")
cs = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cs)
real_read = cs.read

# (name, files edited, old, new): the first two edit the draft too, so only
# the SPOKEN replay can catch them; the third edits production alone.
BOTH = ("script.tsv", "docs/script-draft.tsv")
CONTROLS = [
    ("number word with no SPOKEN row", BOTH, "guard one vault.", "guard one vault, and two doors."),
    ("SPOKEN phrase no longer said", BOTH, "two trustees can be away", "a pair of trustees can be away"),
    ("production line drifts from the draft", ("script.tsv",), "simplest plan", "plainest plan"),
]

caught = 0
for name, rels, old, new in CONTROLS:
    def mutated(path, rels=rels, old=old, new=new):
        text = real_read(path)
        if path in rels:
            assert text.count(old) == 1, (path, old)
            text = text.replace(old, new)
        return text

    cs.read = mutated
    print(f"--- control: {name}")
    ok = cs.source_checks(None)
    cs.read = real_read
    print(f"--- {'NOT CAUGHT' if ok else 'caught'}: {name}")
    caught += not ok

print(f"== {caught} of {len(CONTROLS)} controls caught")
sys.exit(0 if caught == len(CONTROLS) else 1)
