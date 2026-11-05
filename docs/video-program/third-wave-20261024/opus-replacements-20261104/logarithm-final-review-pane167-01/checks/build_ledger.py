"""Read-only: rebuild the viewed-sheets ledger and re-check the hash bindings.

Run from the repository root. OUT must not exist.
    python3 -I -B <this file> OUT

It reads verification.json, the master, its captions, every sheet and every picture, plus sheet-notes.tsv beside this
file. It writes only OUT. The notes are one reader's record of looking at each sheet; this script cannot check them.
"""
import hashlib
import json
import sys
from pathlib import Path

L = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027")
V = L / "logarithm-stage09-final-verify02"
R = L / "logarithm-stage09-render-work02"
NOTES = Path(__file__).resolve().parent / "sheet-notes.tsv"
FIRST, LAST = 1, 112
OUT = Path(sys.argv[1])


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


v = json.loads((V / "verification.json").read_text())
out = []


def check(name, got, want):
    assert got == want, (name, got, want)
    out.append(f"{name}\t{got}\tmatches verification.json")


out.append(f"verification.json\t{sha(V / 'verification.json')}\tread as the binding")
check("master", sha(R / "logarithm_mysteries_master.mp4"), v["sha256"])
check("srt", sha(R / "logarithm_mysteries.en.srt"), v["srt_sha256"])
check("vtt", sha(R / "logarithm_mysteries.en.vtt"), v["vtt_sha256"])
names = [Path(s).name for s in v["sheets"]]
assert names == [f"sheet-{i:03d}.jpg" for i in range(1, len(names) + 1)]
assert not (V / "sheet-000.jpg").exists()
assert all(sha(V / n) == h for n, h in zip(names, v["sheet_sha256"]))
out.append(f"sheets\t{len(names)}\tall hashes match")
assert all(sha(V / Path(p["path"]).name) == p["sha256"] for p in v["pictures"])
out.append(f"pictures\t{len(v['pictures'])}\tall hashes match")

notes = {}
for line in NOTES.read_text().splitlines():
    sheet, status, note = line.split("\t")
    assert status in ("clear", "note")
    notes[int(sheet)] = (status, note)
assert sorted(notes) == list(range(FIRST, LAST + 1))

rows = ["sheet\tsha256\tframes\tseconds\tviewed_by_eye\tstatus\tnote"]
for n in range(FIRST, LAST + 1):
    pics = v["pictures"][(n - 1) * 4:n * 4]
    frames = ",".join(str(p["frame"]) for p in pics)
    secs = ",".join(f"{p['seconds']:.2f}" for p in pics)
    status, note = notes[n]
    rows.append(f"sheet-{n:03d}.jpg\t{v['sheet_sha256'][n - 1]}\t{frames}\t{secs}\tyes\t{status}\t{note}")
out.append(f"ledger rows\t{len(rows) - 1}\tsheet-{FIRST:03d}.jpg to sheet-{LAST:03d}.jpg")

OUT.mkdir(parents=True, exist_ok=False)
(OUT / "viewed-sheets.tsv").write_text("\n".join(rows) + "\n")
(OUT / "binding.txt").write_text("\n".join(out) + "\n")
print("\n".join(out))
