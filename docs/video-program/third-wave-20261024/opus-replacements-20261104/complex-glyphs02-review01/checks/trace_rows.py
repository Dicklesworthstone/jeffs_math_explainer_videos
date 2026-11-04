"""Find the source line of each registry row a finding names.

python3 -I -B trace_rows.py   (from the repo root). Reads the registry and the crate's chapter
sources; the supplement list in chapters/mod.rs is reported apart from the scene modules. Only the
modules chapters/mod.rs declares are searched: older files in that folder are not built.
"""
import re
from pathlib import Path

RUN = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-glyphs02")
CHAPTERS = Path("explainer_corpus/complex_analysis_visual/src/chapters")
GROUPS = {
    "F1 z sub 0": [594, 597, 605, 612, 637, 638, 676, 716, 717],
    "F2 theta": [635, 649, 662, 705],
    "F3 bar minus": [723],
    "F4 Plex superscripts": [18, 242, 342, 411, 475, 503, 508, 509],
    "F5 note, en dash numbers": [512, 513, 514, 515, 516, 517],
    "measure strings": [727, 728],
}


def decode(literal):
    body = literal[1:-1]
    out, i = [], 0
    while i < len(body):
        c = body[i]
        if c != "\\":
            out.append(c)
            i += 1
            continue
        n = body[i + 1]
        if n == "u":
            end = body.index("}", i)
            out.append(chr(int(body[i + 3 : end], 16)))
            i = end + 1
        else:
            out.append({"n": "\n", "t": "\t", "r": "\r", "0": "\0"}.get(n, n))
            i += 2
    return "".join(out)


def main():
    rows = {}
    for number, line in enumerate((RUN / "glyphs/registry.tsv").read_text(encoding="utf-8").splitlines(), 1):
        kind, chapters, literal = line.split("\t", 2)
        rows[number] = (kind, chapters, decode(literal))
    mod_rs = (CHAPTERS / "mod.rs").read_text(encoding="utf-8")
    live = sorted(set(re.findall(r"^\s*(?:pub )?mod (\w+);", mod_rs, re.M)))
    print("modules declared in chapters/mod.rs:", " ".join(live))
    sources = {name: (CHAPTERS / f"{name}.rs").read_text(encoding="utf-8").splitlines() for name in live if (CHAPTERS / f"{name}.rs").exists()}
    sources["mod(supplement)"] = mod_rs.splitlines()
    for group, numbers in GROUPS.items():
        print(f"\n# {group}")
        for n in numbers:
            kind, chapters, text = rows[n]
            print(f"  #{n} {kind} [{chapters}] {text!r}")
            hits = []
            for name, lines in sources.items():
                for i, line in enumerate(lines, 1):
                    if text in line:
                        hits.append(f"{name}:{i}")
            print("     exact literal at: " + (", ".join(hits[:8]) if hits else "none (built by format! or a helper)"))


if __name__ == "__main__":
    main()
