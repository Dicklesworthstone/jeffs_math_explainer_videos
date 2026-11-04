"""Does any pinned source or test already use a space at the head of a script group? Reads only.

python3 -I -B upstream_search.py FRANKEN_MARKDOWN_CHECKOUT FRANKEN_MANIM_CHECKOUT REPO_ROOT

Searches .rs files for the literal forms, with one backslash (raw strings) and with two (ordinary strings). Finding
nothing means no test states this form literally. It does not prove that no test reaches the path some other way.
"""
import sys
from pathlib import Path

BS = chr(92)
FORMS = ["_{" + BS + ",", "^{" + BS + ",", "_{" + BS * 2 + ",", "^{" + BS * 2 + ","]
GROUP = ["{" + BS + ",", "{" + BS * 2 + ","]


def walk(label, top, needles):
    count = files = 0
    for path in sorted(Path(top).rglob("*.rs")):
        if "target" in path.parts:
            continue
        files += 1
        for at, line in enumerate(path.read_text(encoding="utf-8", errors="replace").split("\n"), 1):
            if any(n in line for n in needles):
                count += 1
                print(f"  {label}/{path.relative_to(top)}:{at}: {line.strip()[:90]}")
    print(f"  {label}: {count} lines in {files} .rs files")


def main():
    md, mn, repo = (Path(p) for p in sys.argv[1:4])
    print("A space at the head of a subscript or superscript group:")
    walk("franken_markdown@126a530 fmd-math", md / "fmd-math", FORMS)
    walk("franken_manim@a399d62 fmn-tex", mn / "crates/fmn-tex", FORMS)
    walk("franken_manim@a399d62 fmn-library", mn / "crates/fmn-library", FORMS)
    walk("repo explainer_corpus", repo / "explainer_corpus", FORMS)
    walk("repo crates", repo / "crates", FORMS)
    print("Any group opening with a thin space, in fmd-math:")
    walk("franken_markdown@126a530 fmd-math", md / "fmd-math", GROUP)


if __name__ == "__main__":
    main()
