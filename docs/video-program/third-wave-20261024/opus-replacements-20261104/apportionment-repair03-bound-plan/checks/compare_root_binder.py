#!/usr/bin/env python3
"""Compare this plan with what root's own bind_oliver_plan.py writes on the same source.

    python3 -I -B compare_root_binder.py ROOT_OUTPUT_TOPIC BUNDLE

ROOT_OUTPUT_TOPIC is a disposable copy of the staged topic on which root's binder ran.
Its map is still named oliver03 there, because that binder hard-codes the name.
"""
import json
import sys
from pathlib import Path

root_out, bundle = Path(sys.argv[1]), Path(sys.argv[2])
topic = bundle / "explainer_corpus" / "apportionment"
theirs = json.loads((root_out / "docs" / "narration-direction-map-oliver03.json").read_text())
ours = json.loads((topic / "docs" / "narration-direction-map-oliver04.json").read_text())
assert theirs["scripts"] == ours["scripts"], "scripts differ"
assert theirs["overrides"] == ours["overrides"], "overrides differ"
assert theirs["visual_holds"] == ours["visual_holds"], "hold rows differ"
print(f"root's binder on staged03: {len(theirs['overrides'])} overrides and {len(theirs['visual_holds'])} hold rows, identical to oliver04")
their_units = json.loads((root_out / "narration_units.json").read_text())
our_units = json.loads((topic / "narration_units.json").read_text())
assert their_units["units"] == our_units["units"], "units differ"
assert their_units["script_sha256"] == our_units["script_sha256"], "script binding differs"
print(f"root's binder on staged03: {len(their_units['units'])} units, identical; only the map name, map hash and scope wording differ")
