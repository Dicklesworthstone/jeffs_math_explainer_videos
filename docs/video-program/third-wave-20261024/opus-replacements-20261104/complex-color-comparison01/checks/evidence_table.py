"""Join the hand-typed notes to the images on disk. Writes coverage/evidence.tsv and prints the table for REVIEW.md.

    python3 -I -B .../checks/evidence_table.py     (from the repo root)

Notes are typed by eye from each image. A note for a missing image, or an image with no note, is a failure.
"""
import hashlib
import sys
from pathlib import Path

BUNDLE = Path(__file__).resolve().parent.parent
NOTES = {
    "sheets/z.png": "100%, upper-left quarter. `now` has the dark lattice. Nothing else here shows one at this size.",
    "sheets/z2.png": "`tile` drops the lattice and keeps the stairs. `blend` softens column steps. `rows2`, then `cells2`, shrink the stairs.",
    "sheets/tan.png": "`now` lattice shows most in the flat olive field. Rings only look round under `cells2`.",
    "sheets/sinrecip.png": "Only `cells2` approaches the truth's rings. `tile_p3` goes grey at the core, like truth, in hard blocks.",
    "sheets/sin.png": "Under `tile` and `blend` the near-level band edges step by a whole row. `rows2` and `cells2` smooth them.",
    "sheets/zoom.png": "Long diagonal edges: stepped in `now`, `tile`, `blend`; much less in `rows2`; `cells2` nearly clean.",
    "sheets/recip.png": "**Truth cell blank: a crop bug in my sheet, not the lab** (negative slice on a small plate). Corrected in `evidence/stairs-recip-2x.png`.",
    "sheets/bump.png": "Same ladder. `cells2` is the first rung whose rings read round.",
    "sheets/openbump.png": "Same ladder as `bump`.",
    "evidence/stripes-z-whole.png": "`now` lattice over the whole plate. `overlap`, `overlap06`, `tile`, `tile_raw` clear at 100%. `flat_control` lost its hue along each row, as planted. Black cell at z = 0 in all but the control.",
    "evidence/stripes-z-3x.png": "`overlap06` keeps faint rows at 3x. `overlap`, `tile`, `tile_raw`, `blend` have none. Faint column banding in `tile`, less in `blend`.",
    "evidence/edge-z-corner-8x.png": "`now` and `overlap06` have a dim first row at the plate's top. `tile` and up meet the background cleanly. I find no fault in the harder edge.",
    "evidence/stairs-z2-whole.png": "`cells2` is plainly nearest truth. Under `rows2` near-vertical edges stay soft and centre rings are slightly squashed.",
    "evidence/dense-z2-centre-3x.png": "`now`, `tile`: blocks and a black cell. `blend`: black streak. `tile_p3`: grey cell, stairs stay. `rows2`: rings smear sideways. `cells2`: round to near the centre.",
    "evidence/zero-z-centre-12x.png": "Black cell in `now`, `tile`. `blend` carries it about 6 px along the row in partial shades. `tile_p3`: one grey cell. `rows2`, `cells2`: a small pale run, no black.",
    "evidence/stairs-sinrecip-whole.png": "`cells2` nearest truth outside the core. Core differs from truth under every rung.",
    "evidence/dense-sinrecip-centre-3x.png": "Truth's core is grey. `now`, `tile`, `blend`: false block patterns. `rows2`: horizontal ribbons. `cells2`: false loops, finer. `tile_p3`: grey, in blocks. No rung has it right.",
    "evidence/stairs-sin-whole.png": "`blend`'s long steps on level edges read as directional, as REPAIR.md warned. `rows2` fixes those. `cells2` also rounds the three spirals.",
    "evidence/bands-sin-3x.png": "`tile`, `blend`: whole-row steps on level edges. `tile_p3`: softer colour, same steps. `rows2`, `cells2`: much smaller steps.",
    "evidence/stairs-tan-whole.png": "`now` lattice in flat fields. Rings closest to truth only under `cells2`.",
    "evidence/dense-tan-3x.png": "`rows2` squashes the central rings sideways. `cells2` keeps them round.",
    "evidence/stairs-zoom-whole.png": "Long edges, the plainest stair test: clear gain at `rows2`, again at `cells2`.",
    "evidence/stairs-bump-whole.png": "`cells2` nearest truth. `rows2` good but its waist edges stay softer.",
    "evidence/stairs-openbump-whole.png": "Same as `bump`, film's chapter 14 opening plate.",
    "evidence/stairs-recip-2x.png": "Smallest plate, with truth restored. Same ladder at 2x. Muddiness under `blend` not seen here.",
    "evidence/frame-z-tile.png": "Whole frame: plate only, nothing else on stage. Black cell visible at 100%.",
    "evidence/frame-tan-blend.png": "Whole frame: plate only. Stairs visible at 100% on the ring ends.",
    "evidence/frame-sinrecip-cells2.png": "Whole frame: plate only. Reads well at 100%. Core moire small but present.",
    "evidence/fade-z-2x.png": "`overlap`: bright lines at frames 0, 1, gone at the hold. `overlap06`: fainter lines. `now`: dark rows. `tile`, `blend`: clean at all three.",
    "evidence/fade-tan-2x.png": "`now` lines in the flat field. `tile`, `blend` clean. Stairs are unchanged by fading.",
    "evidence/fade-bump-2x.png": "Same as `tan`, on a busier picture.",
    "evidence/repaint-z2.png": "All three cross cleanly from z^2 to z^3 - 1. `now` keeps its lattice throughout. **My eye read `rows2` as stretched; `checks/repaint_geometry.out` shows it is not.**",
}


def main():
    wanted = sorted(p.relative_to(BUNDLE).as_posix() for d in ("sheets", "evidence") for p in (BUNDLE / d).glob("*.png"))
    parts = sorted(p.relative_to(BUNDLE).as_posix() for p in (BUNDLE / "evidence/lossless").glob("*.png"))
    problems = [f"no note: {w}" for w in wanted if w not in NOTES] + [f"no image: {n}" for n in NOTES if n not in wanted]
    (BUNDLE / "coverage").mkdir(exist_ok=True)
    lines = ["image\tsha256\tnote"]
    print("| Image | sha256 | What I read |\n|---|---|---|")
    for name, note in NOTES.items():
        if name not in wanted:
            continue
        digest = hashlib.sha256((BUNDLE / name).read_bytes()).hexdigest()
        lines.append(f"{name}\t{digest}\t{note}")
        print(f"| `{name}` | `{digest[:12]}` | {note} |")
    for part in parts:
        parent = "evidence/" + Path(part).stem.rsplit("-", 1)[0] + ".png"
        if parent not in NOTES:
            problems.append(f"no parent for {part}")
        digest = hashlib.sha256((BUNDLE / part).read_bytes()).hexdigest()
        lines.append(f"{part}\t{digest}\tExact panels of {parent}; that sheet's note was read here.")
    (BUNDLE / "coverage/evidence.tsv").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"\n{len(NOTES)} notes, {len(wanted)} images, {len(parts)} exact parts")
    for p in problems:
        print("PROBLEM", p)
    sys.exit(1 if problems else 0)


if __name__ == "__main__":
    main()
