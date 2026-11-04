"""Which Rust tests can these edits reach? Reads source text; runs no Rust.

python3 -I -B tests_affected.py SRC_ROOT EDITED_ROOT

Mirrors the checks in the tests that read a changed string or scan a changed file. A mirror shows what the string
comparison should do. It is not a run of the test: Root's Cargo gate decides.
"""
import re
import sys
from pathlib import Path

CRATE = "explainer_corpus/complex_analysis_visual"
CH = CRATE + "/src/chapters/"
FILES = ["closing.rs", "counting.rs", "loop_sums.rs", "mod.rs", "one_derivative.rs", "residues.rs"]
CONSTS = ["LINEAR_TEX", "EACH_TEX", "LIMIT_TEX", "ZERO_TEX", "QUOTIENT_TEX", "SETTLES_TEX", "TIMES_TEX", "LEASH_TEX"]
OLD = ["z_0", r"|-3z|"]
# Every source pattern the scanning tests look for in a changed file, plus the words their logic turns on.
SCAN = [
    "predict(&kit, stage, &line, &ASK", "painter.to(stage, Some(Func::Quintic)", "Some(Func::Quintic)",
    "reading_window(&kit, stage, &line)?;", "let line = say(&kit, stage, ", "wait_until(stage, line.end())?;",
    "fade_out_all", "let exact = ", "let squashed = ", "let panel = ",
]


def text(root, name):
    """A chapter file from root. EDITED_ROOT holds only the six changed files, so the others come from SRC_ROOT."""
    path = Path(root) / CH / name
    if not path.exists():
        path = Path(sys.argv[1]) / CH / name
    return path.read_text(encoding="utf-8")


def tex_list(root):
    """The chapters::TEX supplement, literals and module::CONST references resolved."""
    src = text(root, "mod.rs")
    body = src[src.index("pub const TEX: &[&str] = &["):]
    body = body[:body.index("\n];")]
    out, unresolved = [], []
    for raw in body.splitlines()[1:]:
        item = raw.strip().rstrip(",")
        if not item or item.startswith("//"):
            continue
        lit = re.fullmatch(r'r"(.*)"', item) or re.fullmatch(r'"(.*)"', item)
        ref = re.fullmatch(r"(\w+)::(\w+)", item)
        if lit:
            out.append(lit.group(1))
        elif ref:
            found = re.search(rf'pub const {ref.group(2)}: &str = r?"(.*)";', text(root, ref.group(1) + ".rs"))
            if found:
                out.append(found.group(1))
            else:
                unresolved.append(item)
        else:
            unresolved.append(item)
    return out, unresolved


def tests_in(source):
    """(name, body) for each #[test] function, body running to the next #[test]."""
    parts = re.split(r"#\[test\]", source)[1:]
    out = []
    for part in parts:
        m = re.search(r"fn (\w+)\s*\(", part)
        out.append((m.group(1) if m else "?", part))
    return out


def main():
    src, edited = sys.argv[1], sys.argv[2]

    print("Tests in the changed files that name a changed constant or an old string:")
    total = 0
    for name in FILES:
        for test, body in tests_in(text(src, name)):
            total += 1
            hits = [c for c in CONSTS if re.search(rf"\b{c}\b", body)] + [o for o in OLD if o in body]
            if hits:
                print(f"  {name} {test}: {', '.join(hits)}")
    print(f"  of {total} test functions in the six files")

    leash = re.search(r'pub const LEASH_TEX: &str = r"(.*)";', text(edited, "counting.rs")).group(1)
    print("Mirror of counting.rs the_leash_formula_holds_on_the_unit_circle's string assert:")
    print(f"  {leash!r} contains '\\le 2 < 3 =': {chr(92) + 'le 2 < 3 =' in leash}")

    print("Mirror of mod.rs no_supplement_is_empty_or_duplicated on TEX:")
    for label, root in (("baseline", src), ("edited", edited)):
        items, unresolved = tex_list(root)
        dup = len(items) - len(set(items))
        empty = sum(1 for s in items if not s.strip())
        print(f"  {label}: {len(items)} strings resolved, duplicates {dup}, empty {empty}, "
              f"not resolved by this mirror {len(unresolved)} (array elements and other forms)")
    fresh = ["z_{" + chr(92) + ",0}", "|{-3z}|"]
    crate_text = "".join(p.read_text(encoding="utf-8") for p in (Path(src) / CRATE / "src").rglob("*.rs"))
    edited_text = "".join(text(edited, n) for n in FILES)
    print("  new forms in the baseline crate, so a new string cannot collide with an unresolved one: "
          + ", ".join(f"{f} {crate_text.count(f)}" for f in fresh))
    print("  positive control, the same forms in the six edited files: "
          + ", ".join(f"{f} {edited_text.count(f)}" for f in fresh))
    print("Patterns the source-scanning tests read (counting.rs, mod.rs): counts in the changed files, baseline vs edited")
    changed = 0
    for pattern in SCAN:
        a = sum(text(src, n).count(pattern) for n in FILES)
        b = sum(text(edited, n).count(pattern) for n in FILES)
        changed += a != b
        print(f"  {a:>3} {b:>3}  {pattern}")
    print(f"  patterns whose count changed: {changed}")

    print("Files holding an old string outside the crate (crates, tools, docs/video-program; opus-replacements-20261104 staging excluded):")
    needles = [b"z_0 = 1 + i", b"z_0^2 = 2i", b"|-3z|", b"f(z_0 + h)"]
    found = 0
    for top in ("crates", "tools", "docs/video-program"):
        for path in sorted((Path(src) / top).rglob("*")):
            if not path.is_file() or path.suffix not in {".rs", ".py", ".tsv", ".json", ".md", ".txt"}:
                continue
            if "opus-replacements-20261104" in path.parts or path.stat().st_size > 8_000_000:
                continue
            data = path.read_bytes()
            for needle in needles:
                if needle in data:
                    found += 1
                    print("  ", path.relative_to(src), needle.decode())
    print(f"  hits: {found}")

    print("The same strings elsewhere in the crate (not chapter sources):")
    for rel in ("script.tsv", "narration_units.json", "docs/storyboard.md", "oracle.py"):
        data = (Path(src) / CRATE / rel).read_bytes()
        print(f"  {rel}: " + ", ".join(f"{n.decode()} {data.count(n)}" for n in needles))


if __name__ == "__main__":
    main()
