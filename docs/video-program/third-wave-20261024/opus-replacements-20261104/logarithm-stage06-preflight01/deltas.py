#!/usr/bin/env python3
"""What root's first native run has to look at, from text. Run from the repo root:

    python3 -I -B deltas.py --glyphs glyph-delta.tsv --tests new-tests.tsv

glyph-delta.tsv: string literals in compiled scene code that stage06 adds, per chapter, and any
character no canonical literal already used. A glyph page is what proves a string draws. This only
says where to look first. new-tests.tsv: test functions added or changed by the repairs, with the
exact filtered command. None of these tests has been compiled or run by the author.
"""
import argparse
import importlib.util
import sys
from pathlib import Path

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("stage06_preflight", HERE / "preflight.py")
pf = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = pf
spec.loader.exec_module(pf)
CHAPTERS = list(pf.REPAIRS)


def tokens(path, keep_tests=False):
    toks = pf.lex(Path(path).read_text(encoding="utf-8"))
    return toks if keep_tests else pf.drop_tests(toks)


def strings(path):
    return [t for t in tokens(path) if t.kind == "str"]


def tests_in(path):
    """{function name: its token text} for each #[test]."""
    toks = tokens(path, keep_tests=True)
    found = {}
    for i in range(len(toks) - 3):
        if toks[i].value == "#" and toks[i + 1].value == "[" and toks[i + 2].value == "test" and toks[i + 3].value == "]":
            j = i + 4
            while toks[j].value != "fn":
                j += 1
            name = toks[j + 1].value
            k = j
            while toks[k].value != "{":
                k += 1
            end = pf.match_close(toks, k)
            found[name] = (toks[j].line, repr([(t.kind, t.value) for t in toks[k : end]]))
    return found


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--repo", default=".")
    parser.add_argument("--glyphs", required=True)
    parser.add_argument("--tests", required=True)
    args = parser.parse_args()
    repo = Path(args.repo).resolve()
    stage, canon = repo / pf.STAGE / pf.REL / "src/chapters", repo / pf.REL / "src/chapters"
    known = set()
    for path in canon.glob("*.rs"):
        for t in strings(path):
            known.update(t.value)
    rows = ["chapter\tline\tliteral\tcharacters_no_canonical_literal_used"]
    for name in CHAPTERS:
        before = {t.value for t in strings(canon / name)}
        seen = set()
        for t in strings(stage / name):
            if t.value in before or t.value in seen:
                continue
            seen.add(t.value)
            fresh = "".join(sorted(set(t.value) - known))
            shown = t.value.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n")
            rows.append(f"{name}\t{t.line}\t{shown}\t{fresh}")
    Path(args.glyphs).write_text("\n".join(rows) + "\n", encoding="utf-8")
    out = ["chapter\ttest\tline\tchange\tcommand"]
    for name in CHAPTERS:
        old, new = tests_in(canon / name), tests_in(stage / name)
        for test, (line, body) in sorted(new.items(), key=lambda kv: kv[1][0]):
            if test not in old or old[test][1] != body:
                change = "new" if test not in old else "changed"
                module = name.removesuffix(".rs")
                out.append(f"{name}\t{test}\t{line}\t{change}\tcargo test --locked --offline --manifest-path explainer_corpus/logarithm_mysteries/Cargo.toml chapters::{module}::tests::{test} -- --exact")
    Path(args.tests).write_text("\n".join(out) + "\n", encoding="utf-8")
    print(f"{len(rows) - 1} new literals; {len(out) - 1} new or changed tests")
    return 0


if __name__ == "__main__":
    sys.exit(main())
