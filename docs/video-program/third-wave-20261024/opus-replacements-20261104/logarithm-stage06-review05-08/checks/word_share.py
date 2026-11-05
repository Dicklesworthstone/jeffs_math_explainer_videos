"""Where each phrase this review cites falls inside its cue, as an estimate.

No word-aligned transcript of these previews was used. This spreads the cue's measured length
over its words evenly, and again over its letters, and prints both, so the spread between the
two columns shows how soft a given time is. Reads manifest/cues.tsv only. Standard library.

    python3 -I -B word_share.py
"""
import pathlib
import re

HERE = pathlib.Path(__file__).resolve().parent
PHRASES = {
    "05_20": ["For instance", "the length is two", "so we land on two i"],
    "05_3": ["Watch what that does to a grid", "again and again"],
    "05_6": ["two pi apart"],
    "05_7": ["whole column of angles", "a dial", "the folded slide rule"],
    "06_1": ["walk the number slowly around the circle"],
    "06_2": ["only the angle changes"],
    "06_4": ["Walk the number once around it"],
    "06_6": ["The gap sits at zero until", "passes minus one", "jumps down a lap", "leaps from pi to minus pi", "So that candidate breaks"],
    "06_20": ["stays fixed"],
    "06_7": ["Theirs is a function", "Ours finishes", "So the gap has changed", "That is a contradiction", "No continuous logarithm", "Something has to give"],
    "07_1": ["forbid crossing"],
    "07_2": ["with derivative one over z", "For example", "Log of i", "Log of minus i"],
    "07_4": ["So a path that crosses", "jump by two pi"],
    "07_5": ["The cut is a choice, not a crack", "from minus three halves of pi", "Or cut along a spiral", "without crossing itself", "no loop can wind around zero", "the branch point"],
    "07_6": ["But it jumps", "above minus pi and at most pi", "That is the strip"],
    "08_1": ["three quarters of pi", "Square it", "the angles add to one and a half pi", "But the principal logarithm", "So what angle"],
    "08_2": ["It reports", "So the sum", "while the logarithm of the product", "They differ"],
    "08_3": ["When the sum of the angles", "drops a lap", "dropped a factor of ten", "The rule that survives", "plus a whole number of laps", "It now keeps a lap count", "For our square"],
    "08_4": ["Minus one is", "Write i", "Combine", "That is the root", "So minus one equals one", "Which step broke"],
    "08_5": ["Here a square root", "Each minus one", "so the two roots together", "turn by pi", "point at minus one", "But the product", "Halving the lost lap", "a half turn is a factor of minus one", "a factor of minus one", "The root of a product", "only up to that sign"],
}


def norm(word):
    return re.sub(r"[^a-z0-9']", "", word.lower())


def main():
    print("cue\tphrase\twords_from\twords_to\tletters_from\tletters_to")
    for line in (HERE.parent / "manifest/cues.tsv").read_text(encoding="utf-8").splitlines()[1:]:
        _, cue, start, end, text = line.split("\t")
        if cue not in PHRASES:
            continue
        start, end = float(start), float(end)
        words = text.split()
        keys = [norm(w) for w in words]
        lengths = [len(w) + 1 for w in words]
        total = sum(lengths)
        span = end - start
        for phrase in PHRASES[cue]:
            want = [norm(w) for w in phrase.split()]
            hits = [i for i in range(len(keys) - len(want) + 1) if keys[i : i + len(want)] == want]
            assert hits, (cue, phrase)
            i = hits[0]
            j = i + len(want)
            by_word = (start + span * i / len(words), start + span * j / len(words))
            by_letter = (start + span * sum(lengths[:i]) / total, start + span * sum(lengths[:j]) / total)
            note = "" if len(hits) == 1 else f"\tfirst of {len(hits)}"
            print(f"{cue}\t{phrase}\t{by_word[0]:.1f}\t{by_word[1]:.1f}\t{by_letter[0]:.1f}\t{by_letter[1]:.1f}{note}")


if __name__ == "__main__":
    main()
