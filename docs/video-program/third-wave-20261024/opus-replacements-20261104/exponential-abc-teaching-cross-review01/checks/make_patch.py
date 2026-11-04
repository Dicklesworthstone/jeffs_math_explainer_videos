"""Build patches/phrase-timing.patch and patches/03_7-wording.patch from the frozen candidates.

Usage from the repo root:  python3 -I -B make_patch.py SCRATCH_DIR
Reads the frozen bundle, edits copies in SCRATCH_DIR by exact one-time replacements (any miss stops the run), and
writes the two unified diffs into this review's patches/ folder. Headers are relative to the bundle's candidate/
folder, for `patch -p1`. Nothing else is written in the repo.
"""
import os
import re
import shutil
import subprocess
import sys

R = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
C = R + "exponential-abc-teaching-repair01/candidate/"
O = R + "exponential-abc-teaching-cross-review01/"
S = os.path.abspath(sys.argv[1])


def fn_with_doc(text, name):
    """The whole of a top-level fn, with its doc comment, by brace matching."""
    at = text.index(f"fn {name}(")
    start = text.rfind("\n", 0, at) + 1
    while text[text.rfind("\n", 0, start - 1) + 1 :].startswith("///"):
        start = text.rfind("\n", 0, start - 1) + 1
    depth = 0
    for i in range(at, len(text)):
        if text[i] == "{":
            depth += 1
        elif text[i] == "}":
            depth -= 1
            if depth == 0:
                return text[start : i + 2]
    raise ValueError(name)


def once(text, old, new):
    assert text.count(old) == 1, (text.count(old), old[:70])
    return text.replace(old, new)


def edit(relative, changes):
    src = open(C + relative, encoding="utf-8").read()
    for old, new in changes(src):
        src = once(src, old, new)
    path = os.path.join(S, "b", relative)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as out:
        out.write(src)


def diff(relative_paths):
    out = ""
    for relative in relative_paths:
        os.makedirs(os.path.dirname(os.path.join(S, "a", relative)), exist_ok=True)
        shutil.copyfile(C + relative, os.path.join(S, "a", relative))
        r = subprocess.run(
            ["diff", "-u", "--label", "a/" + relative, "--label", "b/" + relative, "a/" + relative, "b/" + relative],
            cwd=S,
            capture_output=True,
            text=True,
        )
        assert r.returncode == 1, r.stderr
        out += r.stdout
    return out


def factorials(src):
    consts = re.search(r"/// Word for word from 02_4 and 02_6.*?\n/// Each clause of DEBTS_SENTENCE[^\n]*\n", src, re.S)
    return [
        ("use crate::series_model::{\n", "use crate::phrase::until_phrase;\nuse crate::series_model::{\n"),
        ("use explainer_kit::narration::{frames_covering, wait_frames};\n", ""),
        (consts.group(0), "/// Each clause of the debts sentence in 02_6, and the words it puts on screen.\n"),
        (fn_with_doc(src, "until_clause"), ""),
        (
            'until_clause(k, stage, "02_4", 4, OBLIGATIONS_SENTENCE, "and that differentiating")?;',
            'until_phrase(k, stage, "02_4", "and that differentiating")?;',
        ),
        ('until_clause(k, stage, "02_6", 3, DEBTS_SENTENCE, clause)?;\n', 'until_phrase(k, stage, "02_6", clause)?;\n'),
        (
            """    fn every_clause_is_in_its_sentence_in_spoken_order() {
        let at: Vec<usize> = DEBTS.iter().map(|(c, _)| DEBTS_SENTENCE.find(c).unwrap()).collect();
        assert!(at.windows(2).all(|w| w[0] < w[1]));
        assert!(OBLIGATIONS_SENTENCE.contains("and that differentiating"));
    }
""",
            """    fn every_clause_is_in_its_line_in_spoken_order() {
        let script = explainer_kit::Script::parse(crate::phrase::SCRIPT).unwrap();
        let debts = script.text("02_6").unwrap();
        let at: Vec<usize> = DEBTS.iter().map(|(c, _)| debts.find(c).unwrap()).collect();
        assert!(at.windows(2).all(|w| w[0] < w[1]));
        assert!(script.text("02_4").unwrap().contains("and that differentiating"));
    }
""",
        ),
        (
            '"until_clause(k, stage, \\"02_6\\", 3, DEBTS_SENTENCE, clause)?;",',
            'r#"until_phrase(k, stage, "02_6", clause)?;"#,',
        ),
    ]


def product_grid(src):
    consts = re.search(r"/// Word for word from 03_2 and 03_7.*?const LADDER_SENTENCE[^\n]*\n", src, re.S)
    return [
        ("use crate::product_model::{\n", "use crate::phrase::until_phrase;\nuse crate::product_model::{\n"),
        ("use explainer_kit::narration::{frames_covering, wait_frames};\n", ""),
        (consts.group(0), ""),
        (fn_with_doc(src, "until_clause"), ""),
        (
            'until_clause(&k, s, "03_2", 0, ROWS_SENTENCE, "collect them")?;\n        let along',
            'until_phrase(&k, s, "03_2", "collect them")?;\n        let along',
        ),
        (
            'until_clause(&k, s, "03_7", 3, LADDER_SENTENCE, "reciprocals")?;\n        let reciprocal',
            'until_phrase(&k, s, "03_7", "reciprocals")?;\n        let reciprocal',
        ),
        (
            'until_clause(&k, s, "03_7", 3, LADDER_SENTENCE, "continuity")?;\n        let solid',
            'until_phrase(&k, s, "03_7", "continuity")?;\n        let solid',
        ),
        (
            'r#"until_clause(&k, s, "03_2", 0, ROWS_SENTENCE, "collect them")?;"#,',
            'r#"until_phrase(&k, s, "03_2", "collect them")?;"#,',
        ),
        (
            'r#"until_clause(&k, s, "03_7", 3, LADDER_SENTENCE, "reciprocals")?;"#,',
            'r#"until_phrase(&k, s, "03_7", "reciprocals")?;"#,',
        ),
        (
            'r#"until_clause(&k, s, "03_7", 3, LADDER_SENTENCE, "continuity")?;"#,',
            'r#"until_phrase(&k, s, "03_7", "continuity")?;"#,',
        ),
        (
            """    #[test]
    fn the_name_is_earned_in_the_order_the_voice_earns_it() {
""",
            """    #[test]
    fn every_phrase_is_in_its_line_in_spoken_order() {
        let script = explainer_kit::Script::parse(crate::phrase::SCRIPT).unwrap();
        assert!(script.text("03_2").unwrap().contains("collect them"));
        let ladder = script.text("03_7").unwrap();
        assert!(ladder.find("reciprocals").unwrap() < ladder.find("continuity").unwrap());
    }
    #[test]
    fn the_name_is_earned_in_the_order_the_voice_earns_it() {
""",
        ),
    ]


def wording(_src):
    return [
        ("Now we can name it. Call E of one by the letter e.", "Now we can name a number. Call E of one by the small letter e."),
    ]


edit("series01/factorials.rs", factorials)
edit("product01/product_grid.rs", product_grid)
edit("product01/script-slice.tsv", wording)
os.makedirs(O + "patches", exist_ok=True)
with open(O + "patches/phrase-timing.patch", "w", encoding="utf-8") as out:
    out.write(diff(["series01/factorials.rs", "product01/product_grid.rs"]))
with open(O + "patches/03_7-wording.patch", "w", encoding="utf-8") as out:
    out.write(diff(["product01/script-slice.tsv"]))
print("patches written")
