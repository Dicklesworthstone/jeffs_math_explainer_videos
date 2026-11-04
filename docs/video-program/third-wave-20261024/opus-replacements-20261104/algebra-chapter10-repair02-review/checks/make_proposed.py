"""Write proposed-cyclic.rs and proposed.patch from root's frozen candidate.

Run from the repo root: python3 -I -B <this file>
Every edit must match exactly once in the candidate, or nothing is written.
Only this review folder is written.
"""
import difflib
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
O = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104")
CANDIDATE = O / "algebra-chapter10-picture-repair02/cyclic.rs"
LABEL = "explainer_corpus/abstract_algebra_expanded/src/cyclic.rs"

EDITS = [
    # New phrases, in the file's existing style.
    (
        'const EVERY_PHRASE: &str = "and every element of H";\n'
        "#[cfg(test)]\n"
        "const PHRASES: [(&str, usize, &str); 6] = [\n",
        'const EVERY_PHRASE: &str = "and every element of H";\n'
        'const THEOREM_TAIL_PHRASE: &str = "of a cyclic group";\n'
        'const RECAP_WALK_PHRASE: &str = "repeating a move";\n'
        'const RECAP_ORDER_PHRASE: &str = "measures its order";\n'
        'const RECAP_PREDICT_PHRASE: &str = "predicts that return";\n'
        'const RECAP_DIVIDE_PHRASE: &str = "divide exponents";\n'
        'const RECAP_STEP_PHRASE: &str = "the smallest positive step";\n'
        "#[cfg(test)]\n"
        "const PHRASES: [(&str, usize, &str); 12] = [\n",
    ),
    (
        '    ("10_7", 4, SMALLEST_PHRASE),\n',
        '    ("10_7", 0, THEOREM_TAIL_PHRASE),\n'
        '    ("10_7", 4, SMALLEST_PHRASE),\n',
    ),
    (
        '    ("10_9", 2, EVERY_PHRASE),\n];\n',
        '    ("10_9", 2, EVERY_PHRASE),\n'
        '    ("10_11", 0, RECAP_WALK_PHRASE),\n'
        '    ("10_11", 0, RECAP_ORDER_PHRASE),\n'
        '    ("10_11", 1, RECAP_PREDICT_PHRASE),\n'
        '    ("10_11", 2, RECAP_DIVIDE_PHRASE),\n'
        '    ("10_11", 2, RECAP_STEP_PHRASE),\n'
        "];\n",
    ),
    # F3: the line is up before the invitation, and the title reads alone first.
    (
        "        // Build the exponent line during the invitation to see the proof.\n"
        "        let (line, numbers) = exponent_line(stage, &k)?;\n",
        "        // The title reads alone, then the line is up before the invitation to see the proof.\n"
        '        until_phrase(stage, &k, "10_7", 0, THEOREM_TAIL_PHRASE)?;\n'
        "        let (line, numbers) = exponent_line(stage, &k)?;\n",
    ),
    # F2: each 10_10 result waits for the sentence that says it.
    (
        '        k.until_sentence(stage, "10_10", 2)?;\n'
        "        fade(stage, &[squares], true)?;\n"
        '        k.until_sentence(stage, "10_10", 3)?;\n'
        "        fade(stage, &[no], true)?;\n",
        '        k.until_sentence(stage, "10_10", 3)?;\n'
        "        fade(stage, &[squares], true)?;\n"
        '        k.until_sentence(stage, "10_10", 5)?;\n'
        "        fade(stage, &[no], true)?;\n",
    ),
    # F1: the path closes on the first return; the order waits for its words.
    (
        "        d4g::show(stage, &[recap_dial.group], 0.4)?;\n"
        "        draw(stage, &[recap_path], 1.2, 0.0)?;\n"
        "        d4g::show(stage, &[recap_order], 0.4)?;\n",
        "        d4g::show(stage, &[recap_dial.group], 0.4)?;\n"
        '        until_phrase(stage, &k, "10_11", 0, RECAP_WALK_PHRASE)?;\n'
        "        // Slow enough that the triangle closes as the voice reaches the first return.\n"
        "        draw(stage, &[recap_path], 2.2, 0.0)?;\n"
        '        until_phrase(stage, &k, "10_11", 0, RECAP_ORDER_PHRASE)?;\n'
        "        d4g::show(stage, &[recap_order], 0.4)?;\n",
    ),
    # F1 and F5: the pulse waits for "predicts", and leaves the path on its ticks.
    (
        "        d4g::show(stage, &[recap_formula], 0.4)?;\n"
        "        pulse(stage, &[recap_path, recap_order])?;\n",
        "        d4g::show(stage, &[recap_formula], 0.4)?;\n"
        '        until_phrase(stage, &k, "10_11", 1, RECAP_PREDICT_PHRASE)?;\n'
        "        pulse(stage, &[recap_order])?;\n",
    ),
    # F1 and F4: the chunks and ring wait for their words; no pulse on "Next".
    (
        "        d4g::show(stage, &recap_proof, 0.4)?;\n"
        "        draw(stage, &whole, 0.4, 0.4)?;\n"
        "        pulse(stage, &[d_ring])?;\n"
        '        k.until_sentence(stage, "10_11", 3)?;\n'
        "        // Keep the positions visible as the next chapter turns moves into permutations.\n"
        "        pulse(stage, &members)?;\n"
        "        k.finish(stage)\n",
        "        d4g::show(stage, &recap_proof, 0.4)?;\n"
        '        until_phrase(stage, &k, "10_11", 2, RECAP_DIVIDE_PHRASE)?;\n'
        "        draw(stage, &whole, 0.4, 0.4)?;\n"
        '        until_phrase(stage, &k, "10_11", 2, RECAP_STEP_PHRASE)?;\n'
        "        pulse(stage, &[d_ring])?;\n"
        "        k.finish(stage)\n",
    ),
    # F6: guard the hooks' sentence indices.
    (
        '            ("10_10", 6),\n        ] {\n',
        '            ("10_10", 6),\n            ("10_11", 4),\n        ] {\n',
    ),
]

text = CANDIDATE.read_text(encoding="utf-8")
new = text
for old, replacement in EDITS:
    count = new.count(old)
    if count != 1:
        raise SystemExit(f"edit matches {count} times, nothing written:\n{old}")
    new = new.replace(old, replacement)

patch = "".join(
    difflib.unified_diff(
        text.splitlines(keepends=True),
        new.splitlines(keepends=True),
        fromfile=f"a/{LABEL}",
        tofile=f"b/{LABEL}",
        n=3,
    )
)
(HERE / "proposed-cyclic.rs").write_text(new, encoding="utf-8")
(HERE / "proposed.patch").write_text(patch, encoding="utf-8")
added = sum(1 for l in patch.splitlines() if l.startswith("+") and not l.startswith("+++"))
removed = sum(1 for l in patch.splitlines() if l.startswith("-") and not l.startswith("---"))
print(f"{len(EDITS)} edits, +{added} -{removed} lines")
