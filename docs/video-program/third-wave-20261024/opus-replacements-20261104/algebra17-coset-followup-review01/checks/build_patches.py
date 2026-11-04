"""Stage the corrections against root's candidate02, by exact replacement. Run from the repo root.

Writes ../patches/*.patch and nothing else. It never touches candidate02, the stage or canonical.
"""

import difflib
import pathlib

HERE = pathlib.Path(__file__).resolve().parent
BUNDLE = HERE.parent
T = pathlib.Path("narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/applications-repair-stage01")
REL = "explainer_corpus/abstract_algebra_expanded/src/applications.rs"
SCRIPT_REL = "explainer_corpus/abstract_algebra_expanded/script.tsv"

source = (T / "candidate02/applications.rs").read_text(encoding="utf-8")
script = (T / "candidate02/script.tsv").read_text(encoding="utf-8")


def once(text, old, new):
    if text.count(old) != 1:
        raise SystemExit(f"{text.count(old)} of: {old[:70]}")
    return text.replace(old, new)


def write_patch(name, rel, before, after):
    rows = difflib.unified_diff(
        before.splitlines(keepends=True),
        after.splitlines(keepends=True),
        fromfile=f"a/{rel}",
        tofile=f"b/{rel}",
        n=3,
    )
    (BUNDLE / "patches").mkdir(exist_ok=True)
    (BUNDLE / "patches" / name).write_text("".join(rows), encoding="utf-8")


# A. hH=H stays up. It is the only on-screen binding of h and the reason for the second equals sign.
A_OLD = '''        k.until_sentence(stage, "17_13", 3)?;
        fade(stage, &[c], false)?;
        let same_coset = k.tex(stage, r"(gh)H=g(hH)=gH", 32., Palette::green(), &[], v(2., -1.))?;
'''
A_NEW = '''        k.until_sentence(stage, "17_13", 3)?;
        let same_coset = k.tex(stage, r"(gh)H=g(hH)=gH", 32., Palette::green(), &[], v(2., -1.75))?;
'''
write_patch("keep-the-step-in-view.patch", REL, source, once(source, A_OLD, A_NEW))

# B. The paused viewer reads the prompt, not the voice. Name the side and the product.
B_OLD = '"3. Why does changing representative within a coset keep it?",'
B_NEW = '"3. Why does gh name the same left coset as g, when h is in H?",'
write_patch("prompt-names-the-side.patch", REL, source, once(source, B_OLD, B_NEW))

# C. An out-of-range until_sentence returns Ok and does nothing, so pin these six waits.
C_OLD = '''    #[test]
    fn worked_corruptions() {
'''
C_NEW = '''    #[test]
    fn quiz_and_answer_waits_land_on_their_sentences() {
        use crate::axioms::{line_text, sentence_ranges};
        let waited: [(&str, usize, &str); 6] = [
            ("17_12", 1, "Find the subgroup"),
            ("17_12", 2, "Decide whether"),
            ("17_12", 3, "Finally"),
            ("17_13", 1, "The four-clock"),
            ("17_13", 2, "And multiplying"),
            ("17_13", 3, "Now apply"),
        ];
        for (cue, j, opening) in waited {
            let text = line_text(cue).expect("cue in script");
            let (from, to) = sentence_ranges(text)[j];
            assert!(text[from..to].starts_with(opening), "{cue} sentence {j}: {opening}");
        }
    }
    #[test]
    fn worked_corruptions() {
'''
write_patch("waits-are-pinned.test.patch", REL, source, once(source, C_OLD, C_NEW))

# W1. The voice names the side at question time. Reporting only: root decides and binds.
W1_OLD = "Finally, explain why multiplying a coset representative on the right by a subgroup element does not change that coset."
W1_NEW = "Finally, pick a left coset and explain why multiplying its representative on the right by a subgroup element does not change that coset."
write_patch("left-coset-wording.script.patch", SCRIPT_REL, script, once(script, W1_OLD, W1_NEW))

print("patches written")
