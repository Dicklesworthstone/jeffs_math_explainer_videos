"""Build the two correction layers on top of RubyRaven's proposed-applications.rs.

Run from the repo root with python3 -I -B, naming an output directory. Every edit must
match exactly once, or nothing is written. RubyRaven's bundle is only read.

Layer C1 corrects what the patch makes false or leaves unguarded.
Layer C2 moves four early beats onto their words. It needs C1 only for context lines.
To drop one edit, delete its entry and rebuild; each entry is labelled.
"""
import hashlib
import sys
from pathlib import Path

SOURCE = Path(
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
    "algebra-picture-review-17_applications/proposed-applications.rs"
)
PROPOSED_SHA = "686c22730990b045e55253ad86fed582bd80b9b2c2e17e128ca17ed0319985bc"

C1 = [
    (
        "C1b 17_4: the verdict leaves with the digit it no longer describes",
        """        // Sentence 0 changes the symbol and nothing else. The replaced handles leave
        // `digits` and `products`, so the wipe below cannot re-root them: fading a
        // removed mobject out shows it at full opacity first.
        fade(stage, &[digits[3]], false)?;
""",
        """        // Sentence 0 changes the symbol and retires the verdict, which no longer holds. The
        // replaced handles leave `digits` and `products`, so the wipe below cannot re-root
        // them: fading a removed mobject out shows it at full opacity first.
        fade(stage, &[digits[3], valid], false)?;
""",
    ),
    (
        "C1b 17_4: the verdict is no longer in this list",
        "        fade(stage, &[products[3], total, valid], false)?;\n",
        "        fade(stage, &[products[3], total], false)?;\n",
    ),
    (
        "C1a stack heading: true of the received word while the shown word changes",
        """    let heading = k.text(stage, "positions holding a 1", 22., Palette::dim(), v(3.2, 2.8))?;
""",
        """    // Names the received word: the word on screen is repaired or miscorrected under it.
    let heading = k.text(stage, "1s in the received word", 22., Palette::dim(), v(3.2, 2.8))?;
""",
    ),
    (
        "C1c rows_are: the count comes from the code, like the rest of this film",
        """        "each row: a coset of 16 words",
""",
        """        &format!("each row: a coset of {} words", codes.len()),
""",
    ),
    (
        "C1d test: what F2 rests on",
        """        assert_ne!(grid_place(decoded).1, grid_place(SENT).1);
""",
        """        assert_ne!(grid_place(decoded).1, grid_place(SENT).1);
        // 17_18 shows the parity bit beside the received word. Parity holds there and would
        // break beside the miscorrected word, so the word has to return first.
        assert_eq!(SENT.count_ones() % 2, 0);
        assert_eq!(received.count_ones() % 2, 0);
        assert_eq!(decoded.count_ones() % 2, 1);
        // The ring's return runs straight along the decoded word's column.
        assert_eq!(grid_place(received), (7, grid_place(decoded).1));
""",
    ),
]

C2 = [
    (
        "C2 phrases",
        """/// The swap's change appears when the voice reaches it.
const SWAP_CHANGE_PHRASE: &str = "changes the total";
""",
        """/// The digits cross when the voice reaches the exchange.
const SWAP_PHRASE: &str = "exchanging the values";
/// The label column takes its name when the voice gives it.
const SYNDROME_PHRASE: &str = "called the syndrome";
/// The extra bit's two cases, each as the voice reaches it.
const ONE_FLIP_PHRASE: &str = "correct one flipped bit";
const TWO_FLIPS_PHRASE: &str = "while reporting two";
""",
    ),
    (
        "C2d 17_5: the rows with their labels at sentence 3",
        "        fade(stage, &[caption, change_label, product_label], true)?;\n",
        "        fade(stage, &[change_label, product_label], true)?;\n",
    ),
    (
        "C2d 17_5: the conclusion with its reason at sentence 4",
        "        fade(stage, &[condition], true)?;\n",
        "        fade(stage, &[caption, condition], true)?;\n",
    ),
    (
        "C2c 17_7: the digits cross on the words",
        """        fade(stage, &[before], true)?;
        // Separate vertical lanes keep the crossing digits legible.
""",
        """        fade(stage, &[before], true)?;
        until_phrase(stage, &k, "17_7", 1, SWAP_PHRASE)?;
        // Separate vertical lanes keep the crossing digits legible.
""",
    ),
    (
        "C2c 17_7: the crossing now reaches 'by the product' on its own",
        """        until_phrase(stage, &k, "17_7", 1, SWAP_CHANGE_PHRASE)?;
""",
        "",
    ),
    (
        "C2b 17_16: the name when the voice gives it, after the stack",
        """    let syndrome_name = k.text(stage, "syndrome", 22., Palette::dim(), v(4.95, 0.1))?;
    fade(stage, &labels, true)?;
    fade(stage, &[syndrome_name], true)?;
    let (mut stack, mut result) = syndrome_stack(stage, k, word.value)?;
""",
        """    fade(stage, &labels, true)?;
    let (mut stack, mut result) = syndrome_stack(stage, k, word.value)?;
    until_phrase(stage, k, "17_16", 1, SYNDROME_PHRASE)?;
    let syndrome_name = k.text(stage, "syndrome", 22., Palette::dim(), v(4.95, 0.1))?;
    fade(stage, &[syndrome_name], true)?;
""",
    ),
    (
        "C2a 17_18: box, one flip, then the word returns as the voice reaches two",
        """    k.until_sentence(stage, "17_18", 1)?;
    // Back to the two-flip word, the case the extra bit reports: four ones, parity bit
    // zero, so parity holds while the label is not 000. The miscorrected word has three.
    word.flip(stage, k, miscorrect, Palette::ink())?;
    d4g::glide(stage, ring, grid_at(landed.0, landed.1), 0.6)?;
    fade(stage, &[sent_column, wrong_column, sent_name, wrong_name], false)?;
    let parity = frame(stage, (box_x(7) + 0.95, WORD_Y), (0.6, 0.7), Palette::purple(), 3.0)?;
    let parity_name = k.text(
        stage,
        "parity",
        24.,
        Palette::purple(),
        v(box_x(7) + 0.95, WORD_Y - 0.6),
    )?;
    let parity_bit = k.text(
        stage,
        &(SENT.count_ones() % 2).to_string(),
        36.,
        Palette::purple(),
        v(box_x(7) + 0.95, WORD_Y),
    )?;
    fade(stage, &[parity, parity_name, parity_bit], true)?;
    let one_flip = k.text(
        stage,
        "One flip: parity breaks, so repair it",
        24.,
        Palette::purple(),
        v(-1.8, 0.55),
    )?;
    fade(stage, &[one_flip], true)?;
    let two_flips = k.text(
        stage,
        "Two flips: parity holds, label not 000, so report",
        24.,
        Palette::purple(),
        v(-1.8, 0.05),
    )?;
    fade(stage, &[two_flips], true)?;
    k.until_sentence(stage, "17_18", 2)?;
""",
        """    k.until_sentence(stage, "17_18", 1)?;
    let parity = frame(stage, (box_x(7) + 0.95, WORD_Y), (0.6, 0.7), Palette::purple(), 3.0)?;
    let parity_name = k.text(
        stage,
        "parity",
        24.,
        Palette::purple(),
        v(box_x(7) + 0.95, WORD_Y - 0.6),
    )?;
    fade(stage, &[parity, parity_name], true)?;
    until_phrase(stage, k, "17_18", 1, ONE_FLIP_PHRASE)?;
    let one_flip = k.text(
        stage,
        "One flip: parity breaks, so repair it",
        24.,
        Palette::purple(),
        v(-1.8, 0.55),
    )?;
    fade(stage, &[one_flip], true)?;
    // Back to the two-flip word, the case the extra bit reports: four ones, parity bit
    // zero, so parity holds while the label is not 000. The miscorrected word has three,
    // so the bit stays off screen until the word has returned.
    until_phrase(stage, k, "17_18", 1, TWO_FLIPS_PHRASE)?;
    word.flip(stage, k, miscorrect, Palette::ink())?;
    d4g::glide(stage, ring, grid_at(landed.0, landed.1), 0.6)?;
    fade(stage, &[sent_column, wrong_column, sent_name, wrong_name], false)?;
    let parity_bit = k.text(
        stage,
        &(SENT.count_ones() % 2).to_string(),
        36.,
        Palette::purple(),
        v(box_x(7) + 0.95, WORD_Y),
    )?;
    let two_flips = k.text(
        stage,
        "Two flips: parity holds, label not 000, so report",
        24.,
        Palette::purple(),
        v(-1.8, 0.05),
    )?;
    fade(stage, &[parity_bit, two_flips], true)?;
    k.until_sentence(stage, "17_18", 2)?;
""",
    ),
    (
        "C2 test: every waited phrase",
        """        let waited: [(&str, usize, &str); 2] = [("17_3", 1, CONTRIBUTIONS_PHRASE), ("17_7", 1, SWAP_CHANGE_PHRASE)];
""",
        """        let waited: [(&str, usize, &str); 5] = [
            ("17_3", 1, CONTRIBUTIONS_PHRASE),
            ("17_7", 1, SWAP_PHRASE),
            ("17_16", 1, SYNDROME_PHRASE),
            ("17_18", 1, ONE_FLIP_PHRASE),
            ("17_18", 1, TWO_FLIPS_PHRASE),
        ];
""",
    ),
]


def apply(text, edits):
    for label, old, new in edits:
        count = text.count(old)
        if count != 1:
            raise SystemExit(f"{label}: matched {count} times, wanted 1")
        text = text.replace(old, new)
    return text


def main():
    out = Path(sys.argv[1])
    raw = SOURCE.read_bytes()
    if hashlib.sha256(raw).hexdigest() != PROPOSED_SHA:
        raise SystemExit("proposed-applications.rs is not the reviewed file")
    base = raw.decode("utf-8")
    c1 = apply(base, C1)
    c2 = apply(c1, C2)
    for name, text in (("C1", c1), ("C1C2", c2)):
        path = out / name
        path.mkdir(parents=True, exist_ok=True)
        (path / "applications.rs").write_text(text, encoding="utf-8")
        print(name, hashlib.sha256(text.encode()).hexdigest())


if __name__ == "__main__":
    main()
