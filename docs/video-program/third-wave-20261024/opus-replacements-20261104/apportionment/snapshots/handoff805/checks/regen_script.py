"""Regenerate the candidate's script.tsv with tools/storyboard.py's own
parse/check/script_text, without running its main (which writes canonical
script.tsv files). argv: repo root, candidate dir. Writes only the candidate's
script.tsv; prints the report and any problems."""

import runpy
import sys
from pathlib import Path

root, cand = Path(sys.argv[1]), Path(sys.argv[2])
mod = runpy.run_path(str(root / "tools" / "storyboard.py"), run_name="storyboard_lib")
chapters = mod["parse"](cand / "docs" / "storyboard.md")
problems, report = mod["check"]("apportionment", chapters)
print("\n".join(report))
text = mod["script_text"]("apportionment", chapters)
out = cand / "script.tsv"
old = out.read_text() if out.is_file() else ""
if old != text:
    out.write_text(text)
    print(f"wrote script.tsv ({text.count(chr(10)) - 1} lines)")
else:
    print("script.tsv unchanged")
for p in problems:
    print("PROBLEM", p)
print("problems:", len(problems))
