#!/usr/bin/env python3
"""Point the lab copy's two path dependencies at the kit copy, and refuse anything unexpected.

The lab copy's manifest names the canonical crates by absolute path, so without this the isolated
build would compile the canonical kit, which has no budget wiring.

    python3 -I -B repoint_lab.py LAB VIEW
"""
import re
import sys
from pathlib import Path

lab, view = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
manifest = lab / "Cargo.toml"
text = manifest.read_text()
for name in ("explainer_kit", "explainer_score"):
    if not (view / "crates" / name / "Cargo.toml").is_file():
        sys.exit(f"the kit copy has no crates/{name}")
    text, count = re.subn(
        rf'^{name} = \{{ path = "[^"]*" \}}$',
        f'{name} = {{ path = "{view}/crates/{name}" }}',
        text,
        flags=re.M,
    )
    if count != 1:
        sys.exit(f"expected one {name} path dependency in the lab manifest, found {count}")
manifest.write_text(text)
print("the lab copy now builds against the kit copy: explainer_kit, explainer_score")
