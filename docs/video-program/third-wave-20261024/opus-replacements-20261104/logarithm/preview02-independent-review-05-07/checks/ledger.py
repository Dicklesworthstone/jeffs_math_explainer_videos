"""Scratch only: exact sheet ledger for the review, one row per sheet viewed."""
import hashlib
import json
import pathlib
import sys

B = pathlib.Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/logarithm-opus-rebuild-frames02")
out = pathlib.Path(sys.argv[1])
rows = ["chapter\tsheet\tfirst_frame\tlast_frame\tfirst_t\tlast_t\tsha256"]
tot_s = tot_f = 0
for ch in ("05_complex_turns", "06_lifting_loops", "07_branch_cut"):
    d = B / ch
    t = json.load(open(d / "sampling.json"))["times"]
    sheets = sorted(d.glob("sheet-*.png"))
    assert len(sheets) == (len(t) + 3) // 4, (ch, len(sheets), len(t))
    for k, p in enumerate(sheets):
        assert p.name == f"sheet-{k:03d}.png"
        a, b = 4 * k, min(4 * k + 3, len(t) - 1)
        rows.append(f"{ch}\t{k:03d}\t{a}\t{b}\t{t[a]:.3f}\t{t[b]:.3f}\t{hashlib.sha256(p.read_bytes()).hexdigest()}")
    print(ch, len(sheets), "sheets", len(t), "frames")
    tot_s += len(sheets)
    tot_f += len(t)
out.write_text("\n".join(rows) + "\n")
print("total", tot_s, "sheets", tot_f, "frames")
