"""Print the pinned engine lines HANDOFF.md cites, with file hashes.

python3 -I -B pinned_excerpts.py FRANKEN_MARKDOWN_CHECKOUT FRANKEN_MANIM_CHECKOUT

The checkouts are franken_markdown rev 126a530 and franken_manim rev a399d62.
"""
import hashlib
import sys
from pathlib import Path

MD = "franken_markdown@126a530"
MN = "franken_manim@a399d62"
RANGES = [
    (MD, "fmd-math/src/parse.rs", [(168, 174), (311, 315), (345, 352), (358, 365), (851, 864), (886, 895), (1347, 1351), (1378, 1390)]),
    (MD, "fmd-math/src/node.rs", [(212, 219), (231, 238)]),
    (MD, "fmd-math/src/style.rs", [(41, 47)]),
    (MD, "fmd-math/src/atom.rs", [(205, 222), (228, 247), (281, 322)]),
    (MD, "fmd-math/src/layout.rs", [(203, 205), (309, 311), (337, 343), (377, 390), (470, 491), (826, 834), (1150, 1160), (1172, 1183), (1196, 1228)]),
    (MD, "fmd-math/src/mbox.rs", [(313, 314)]),
    (MD, "fmd-math/src/faces.rs", [(148, 149)]),
    (MD, "fmd-math/src/commands.rs", [(680, 681), (935, 936), (953, 953)]),
    (MN, "crates/fmn-library/src/tex.rs", [(172, 180)]),
    (MN, "crates/fmn-tex/src/engine.rs", [(534, 540)]),
]


def main():
    roots = {MD: Path(sys.argv[1]), MN: Path(sys.argv[2])}
    for repo, rel, spans in RANGES:
        data = (roots[repo] / rel).read_bytes()
        lines = data.decode("utf-8").split("\n")
        print(f"=== {repo} {rel}  sha256 {hashlib.sha256(data).hexdigest()}")
        for a, b in spans:
            for n in range(a, b + 1):
                print(f"{n:>5}  {lines[n - 1]}")
            print("  ...")


if __name__ == "__main__":
    main()
