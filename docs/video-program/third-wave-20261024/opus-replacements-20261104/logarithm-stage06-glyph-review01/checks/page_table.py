"""Join the hand-typed viewing notes to the measured page map. Writes coverage/page-evidence.tsv and prints the Markdown table.

    python3 -I -B .../checks/page_table.py     (from the repo root)

Notes are typed by eye. Hashes, row ranges and ink-row counts come from coverage/page-map.tsv, so they cannot drift from the
pages. Every row a note names (r377) must sit on that note's page, which catches a note filed one page off.
"""
import csv
import hashlib
import re
import sys
from pathlib import Path

BUNDLE = Path(__file__).resolve().parent.parent
NOTES = {
    0: "Numbers and short labels. r4 `→` is the heavier Noto arrow.",
    1: "Numbers and short labels. Clean.",
    2: "Numbers and short labels. Clean.",
    3: "Numbers and short labels. Clean.",
    4: "Numbers and short labels. Clean.",
    5: "r66 is the supplement, the only row not captured from a chapter: `θ` in its open form, Noto `− × ≈ ≠ ≤ → ↔ ∞`, Plex `½ ₂`.",
    6: "Short labels. Clean.",
    7: "Short labels. Clean.",
    8: "Short labels. `×` heavier than its digits (F5).",
    9: "`2×2`, `2π`, `2πi`: tight, legible.",
    10: "Short labels. Clean.",
    11: "Short labels. Clean.",
    12: "r160–r162 `(−I) × diag(2, ½)`, `diag(−2, −½)`: Noto `−`, Plex `½` (F5).",
    13: "Short labels. Clean.",
    14: "`Log(−i) = −πi/2`. Clean.",
    15: "Short labels. Clean.",
    16: "Short labels. Clean.",
    17: "`−cI`, `near π; … −π`. Clean.",
    18: "r235 breaks mid-word: probe's 64-character chunking, not the film.",
    19: "`π i → −πi, gap 0 → −1`, `|z| = r`. Clean.",
    20: "r254, r258, r259 Plex `½` (F5); r261 `e ≈ 2.718`; `det = (−2) × (−½) = +1`.",
    21: "`(−1) − (−1) = 0`, `±1.5 ×`. Clean.",
    22: "`Log(−1) = iπ`. Clean.",
    23: "r287 `for z²:` Plex `²` heavier than the serif beside it (F5); `height −π`.",
    24: "r302 `i × i = −1: two quarter turns`; r307 `it breaks at −1`. Clean.",
    25: "r313, r315 `→`; r317 `keep −π < turning < π`; r322 `½, 1½, 2½, …` (F5).",
    26: "r323 `→`; r325 ends in `×`; r328 `ln 2 ≈ 0.693`; r329 `×e`. Clean.",
    27: "r335 `mirror: −1`; r346 `×10`. Clean.",
    28: "r350, r353 `×10`; r357 `×4`; r355 `size-2`. Clean.",
    29: "r359 `2πi`; r364–r367 are 64-character chunks; r365 en dash `1146–1151`; r367 `(−∞, 0]`.",
    30: "r370 `∫` is the Noto integral; r372 `⇒`, `(−∞, 0]`; r367 `≤`. Chunk breaks are the probe's.",
    31: "r372 tail `)`; r373 `±½, ±1½, …`, `§4`; r377 `θ` open form, twice (F5); r382 `≈`, `÷`.",
    32: "r393 `−I`; r389 chunk break is the probe's. Clean.",
    33: "r395–r399 `½`, `1½`, `2½` (F5); r400 `0.28π`.",
    34: "r410 `½(I + mirror)` (F5); r406, r407 live-style numbers. Clean.",
    35: "**r420 `the line goes on`: new, correct.** r428 `±π`.",
    36: "r436 `↔`; r438 chunk break is the probe's; r435 `1.380 laps`. Clean.",
    37: "r443 `(−½) − (−2)`; r452 `→`. Clean.",
    38: "r455 `½` alone, in Plex; r456 `× ?`; r466 `×½` (F5).",
    39: "r472–r476 live values with thin en dash, against r479 heavy `−` (F3); r467 `√`; r478 `⇒`; r477 `…` low.",
    40: "r480–r492 fixed-label `−`; r491 `−b`, from `\"-b\"` through `shown()`; r492 `−½`.",
    41: "**r497 `(ℓ)`: new, correct, Noto (F4).** r498 tight unary control; r503 floating colon (F2); r502 `P^{-1}` clears at every ink level.",
    42: "r507–r509 root bars clear by eye; r511 `e^X` clears at every ink level, r510 by eye; r512, r513 floating colon (F2); r514 `⟷`, `\\left(`.",
    43: "r516 `⟺`; r517 `⇒`; r518, r519 `\\frac`; r520 `↔`; r522–r524 `≈`. Clean.",
    44: "r527 `e^{\\ln y}`; r528 `diag`; r530, r531, r533 subscripts on `log`; r532 floating colon (F2).",
    45: "r535 `\\tfrac`; r538 `Arg(−1)`; r542 `≠`; r543 `⟺`, `≤`; r540, r544 clean.",
    46: "r546–r549 root bars and exponents; r547, r549 `\\frac` in an exponent; r550 `×`; r551, r555 `\\tfrac`.",
    47: "**r559 singular S and r565 DIFFER: new, correct.** r560 `so − 1` spacing (F1); r557 floating colon (F2); r558, r563 clean.",
    48: "**r575 `e^{tp}` line: new, correct.** r569 `e^{-\\ln 2+i\\pi}`; r570 `≠`; r574 `e^{tX}`.",
    49: "**r584 `y = x`: new, correct.** r578 `e^{-x}` clears; r579 `\\frac`; r586 floating colon (F2).",
    50: "r587, r588. Last page; the rest of the sheet is empty, as expected.",
}


def main():
    pages = list(csv.DictReader(open(BUNDLE / "coverage/page-map.tsv", encoding="utf-8"), delimiter="\t", quoting=csv.QUOTE_NONE))
    problems = []
    if set(NOTES) != set(range(len(pages))):
        problems.append(f"notes cover {len(NOTES)} of {len(pages)} pages")
    out = ["page\tfile\tsha256\tfirst_registry_row\tlast_registry_row\tink_rows\tnote"]
    table = ["| Page | sha256 | Rows | Ink rows | What I read |", "|---|---|---|---|---|"]
    for i, p in enumerate(pages):
        first, last = int(p["first_registry_row"]), int(p["last_registry_row"])
        note = NOTES[i]
        for r in re.findall(r"\br(\d{1,3})\b", note):
            if not first <= int(r) <= last:
                problems.append(f"page {i}: note names r{r}, page holds {first}-{last}")
        out.append(f"{i}\t{p['page']}\t{p['sha256']}\t{first}\t{last}\t{p['bands']}\t{note}")
        table.append(f"| {i} | `{p['sha256'][:12]}` | {first}–{last} | {p['bands']} | {note} |")
    (BUNDLE / "coverage/page-evidence.tsv").write_text("\n".join(out) + "\n", encoding="utf-8")
    joined = "".join(f"{p['sha256']}  {p['page']}\n" for p in pages)
    print("\n".join(table))
    print(f"\nall {len(pages)} page hashes, sha256 of the sorted list: {hashlib.sha256(joined.encode()).hexdigest()}")
    print(f"ink rows {sum(int(p['bands']) for p in pages)}")
    for problem in problems:
        print("PROBLEM", problem)
    sys.exit(1 if problems else 0)


if __name__ == "__main__":
    main()
