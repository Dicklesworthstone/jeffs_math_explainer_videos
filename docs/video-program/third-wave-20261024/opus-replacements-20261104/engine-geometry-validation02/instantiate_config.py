#!/usr/bin/env python3
"""Write the Cargo --config file for one export. Refuses an existing file or a missing package.

    python3 -I -B instantiate_config.py EXPORT OUT.toml
"""
import re
import sys
from pathlib import Path

export, out = Path(sys.argv[1]).resolve(), Path(sys.argv[2])
template = (Path(__file__).resolve().parent / "engine-budget.config.toml.in").read_text()
paths = re.findall(r'path = "@EXPORT@/([^"]+)"', template)
assert len(paths) == 19, len(paths)
for rel in paths:
    if not (export / rel / "Cargo.toml").is_file():
        sys.exit(f"the export has no {rel}/Cargo.toml")
if not (export / "Cargo.toml").is_file():
    sys.exit("the export has no workspace root, which the packages inherit from")
with out.open("x") as file:
    file.write(template.replace("@EXPORT@", str(export)))
print(f"wrote {out} with {len(paths)} path patches")
