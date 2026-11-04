"""Record that a page was opened at full size, with its hash, when, and what was seen.

python3 -I -B log_page.py PAGE "rows seen" "observation"   (from the repo root)
Appends one line to viewed-log.tsv beside this script.
"""
import datetime
import hashlib
import sys
from pathlib import Path

here = Path(sys.argv[0]).resolve().parent
PAGES = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-glyphs02/glyphs/pages")
page, rows, note = int(sys.argv[1]), sys.argv[2], sys.argv[3]
name = f"frame_{page:06d}.png"
digest = hashlib.sha256((PAGES / name).read_bytes()).hexdigest()
when = datetime.datetime.now(datetime.timezone.utc).strftime("%H:%M:%SZ")
with (here / "viewed-log.tsv").open("a", encoding="utf-8") as f:
    f.write(f"{name}\t{digest}\t{when}\t{rows}\t{note}\n")
print(name, digest[:12], when, rows)
