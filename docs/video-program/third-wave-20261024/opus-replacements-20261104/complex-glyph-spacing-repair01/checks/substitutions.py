"""The exact F1 and F3 edits for complex_analysis_visual, line by line.

python3 -I -B substitutions.py check SRC_ROOT
    Read-only. Confirms each old line sits at its recorded place in SRC_ROOT, and that nothing outside this list
    still carries z_0 in a TeX string or a bar followed by a minus.
python3 -I -B substitutions.py apply SRC_ROOT OUT_ROOT
    Writes edited copies of the six files under OUT_ROOT, keeping repo-relative paths. Refuses an existing file.
    Never writes under SRC_ROOT.
python3 -I -B substitutions.py patch SRC_ROOT OUT_ROOT PATCH_FILE
    Writes a unified diff between SRC_ROOT and OUT_ROOT with repo-relative a/ and b/ names.

SRC_ROOT is the directory holding explainer_corpus/ (the repo root). Each edit replaces a whole line, so a line that
has moved or changed stops the check rather than being edited blind.
"""
import difflib
import hashlib
import re
import sys
from pathlib import Path

CH = "explainer_corpus/complex_analysis_visual/src/chapters/"

# (fix, file, 1-indexed line, exact old line, new lines)
EDITS = [
    ("F1", "closing.rs", 28,
     r'''pub const LIMIT_TEX: &str = r"\lim_{h \to 0}\frac{f(z_0 + h) - f(z_0)}{h}";''',
     [r'''pub const LIMIT_TEX: &str = r"\lim_{h \to 0}\frac{f(z_{\,0} + h) - f(z_{\,0})}{h}";''']),
    ("F3", "counting.rs", 50,
     r'''pub const LEASH_TEX: &str = r"|z^5 + 1| \le 2 < 3 = |-3z|";''',
     [r'''pub const LEASH_TEX: &str = r"|z^5 + 1| \le 2 < 3 = |{-3z}|";''']),
    ("F1", "loop_sums.rs", 50,
     r'''pub const LINEAR_TEX: &str = r"f(z) \approx a + b\,(z - z_0)";''',
     [r'''pub const LINEAR_TEX: &str = r"f(z) \approx a + b\,(z - z_{\,0})";''']),
    ("F1", "loop_sums.rs", 51,
     r'''pub const EACH_TEX: &str = r"\oint (a + b\,(z - z_0))\,dz = 0";''',
     [r'''pub const EACH_TEX: &str = r"\oint (a + b\,(z - z_{\,0}))\,dz = 0";''']),
    ("F1", "mod.rs", 407,
     r'''    r"z_0 = 1 + i",''',
     [r'''    r"z_{\,0} = 1 + i",''']),
    ("F1", "mod.rs", 408,
     r'''    r"z_0^2 = 2i",''',
     [r'''    r"z_{\,0}^2 = 2i",''']),
    ("F1", "one_derivative.rs", 26,
     r'''pub const QUOTIENT_TEX: &str = r"\frac{f(z_0 + h) - f(z_0)}{h}";''',
     [r'''pub const QUOTIENT_TEX: &str = r"\frac{f(z_{\,0} + h) - f(z_{\,0})}{h}";''']),
    ("F1", "one_derivative.rs", 44,
     r'''pub const SETTLES_TEX: &str = r"\to\, f'(z_0)";''',
     [r'''pub const SETTLES_TEX: &str = r"\to\, f'(z_{\,0})";''']),
    ("F1", "one_derivative.rs", 47,
     r'''pub const TIMES_TEX: &str = r"\times\, f'(z_0)";''',
     [r'''pub const TIMES_TEX: &str = r"\times\, f'(z_{\,0})";''']),
    ("F1", "one_derivative.rs", 211,
     r'''                r"z_0 = 1 + i",''',
     [r'''                r"z_{\,0} = 1 + i",''']),
    # The longer literal takes this call past rustfmt's call width, so it goes vertical like its neighbour at 209.
    ("F1", "one_derivative.rs", 217,
     r'''            kit.tex(stage, r"z_0^2 = 2i", 28.0, output(), &[], out.p(w0 + C::new(-0.6, 0.3)))?,''',
     [r'''            kit.tex(''',
      r'''                stage,''',
      r'''                r"z_{\,0}^2 = 2i",''',
      r'''                28.0,''',
      r'''                output(),''',
      r'''                &[],''',
      r'''                out.p(w0 + C::new(-0.6, 0.3)),''',
      r'''            )?,''']),
    ("F1", "residues.rs", 31,
     r'''pub const ZERO_TEX: &str = r"\frac{f'}{f} \approx \frac{m}{z - z_0}";''',
     [r'''pub const ZERO_TEX: &str = r"\frac{f'}{f} \approx \frac{m}{z - z_{\,0}}";''']),
]

RAW = re.compile(r'r"([^"]*)"')


def read_lines(root, name):
    return (Path(root) / CH / name).read_text(encoding="utf-8").split("\n")


def check_groups(source):
    """A port of crates/explainer_kit/src/glyphs.rs check_groups."""
    depth, escaped = 0, False
    for ch in source:
        if escaped:
            escaped = False
            continue
        if ch == "\\":
            escaped = True
        elif ch == "{":
            depth += 1
        elif ch == "}":
            if depth == 0:
                return "unmatched closing TeX group"
            depth -= 1
    return "unclosed TeX group" if depth else "ok"


def check(root):
    bad = 0
    for fix, name, line, old, new in EDITS:
        lines = read_lines(root, name)
        here = lines[line - 1] if line <= len(lines) else None
        count = lines.count(old)
        state = "ok" if here == old and count == 1 else "MISMATCH"
        bad += state != "ok"
        tex_new = next(m.group(1) for text in new for m in [RAW.search(text)] if m)
        tex_old = RAW.search(old).group(1)
        print(f"{state} {fix} {name}:{line} old_groups={check_groups(tex_old)} new_groups={check_groups(tex_new)} "
              f"chars {len(tex_old)}->{len(tex_new)} z_0 {tex_old.count('z_0')}->{tex_new.count('z_0')} "
              f"thin-sub {tex_new.count(chr(95) + '{' + chr(92) + ',0}')}")
        if len(tex_new) >= 64:
            bad += 1
            print("  new string reaches 64 characters")
    # Anything these edits do not list. A z_0 in a raw string, or a bar then a minus inside a raw string.
    listed = {(name, line) for _, name, line, _, _ in EDITS}
    stray = 0
    for path in sorted((Path(root) / CH.rsplit("/chapters/", 1)[0]).rglob("*.rs")):
        for at, text in enumerate(path.read_text(encoding="utf-8").split("\n"), 1):
            key = (path.name, at)
            strings = RAW.findall(text)
            if any("z_0" in s or re.search(r"\|\s*-", s) for s in strings) and key not in listed:
                stray += 1
                print("UNLISTED", path.relative_to(root), at, text.strip())
    print(f"edits {len(EDITS)}; mismatches {bad}; unlisted raw strings {stray}")
    return bad + stray


def edited(root, name):
    lines = read_lines(root, name)
    # Bottom up, so an edit that adds lines leaves the recorded numbers above it good.
    for _, file_name, line, old, new in sorted((e for e in EDITS if e[1] == name), key=lambda e: -e[2]):
        if lines[line - 1] != old:
            raise SystemExit(f"{name}:{line} is not the recorded old line")
        lines[line - 1:line] = new
    return "\n".join(lines)


def names():
    return sorted({e[1] for e in EDITS})


def apply(src, out):
    if Path(src).resolve() == Path(out).resolve():
        raise SystemExit("refusing to write over the source root")
    for name in names():
        target = Path(out) / CH / name
        if target.exists():
            raise SystemExit(f"refusing: {target} exists")
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(edited(src, name), encoding="utf-8")
        print(hashlib.sha256(target.read_bytes()).hexdigest(), CH + name)


def patch(src, out, patch_file):
    with open(patch_file, "w", encoding="utf-8") as f:
        for name in names():
            rel = CH + name
            old = (Path(src) / rel).read_text(encoding="utf-8").splitlines(keepends=True)
            new = (Path(out) / rel).read_text(encoding="utf-8").splitlines(keepends=True)
            f.writelines(difflib.unified_diff(old, new, f"a/{rel}", f"b/{rel}", n=3))


def main():
    mode = sys.argv[1]
    if mode == "check":
        raise SystemExit(1 if check(sys.argv[2]) else 0)
    if mode == "apply":
        apply(sys.argv[2], sys.argv[3])
    elif mode == "patch":
        patch(sys.argv[2], sys.argv[3], sys.argv[4])
    else:
        raise SystemExit(__doc__)


if __name__ == "__main__":
    main()
