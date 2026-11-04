#!/usr/bin/env python3
"""Does each beat of chapter 11 fit its sentence? An estimate, not a measurement.

There are no Oliver takes for chapter 11, so this simulates the scene clock
against the candidate script at three assumed speaking rates. Sentence and
phrase times use the house rule: share of characters. The picture never makes
the voice wait inside a cue, and until_sentence/until_phrase are no-ops once
their moment has passed, so lateness is how far a beat starts after its words.
A cue's overrun is silence before the next k.say.

Beat durations are read by hand from candidate src/permutations.rs.

Run from the repository root: python3 -I -B <this file>
"""
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1] / "candidate" / "explainer_corpus" / "abstract_algebra_expanded"

# (cue, sentence, phrase or None, seconds of animation, what)
BEATS = [
    ("11_1", 0, None, 1.2, "title, cards and slots, legal arrows"),
    ("11_1", 0, "with no collisions", 0.8, "red arrows, ring, caption"),
    ("11_1", 0, "no missing destinations", 0.4, "gold ring, caption"),
    ("11_1", 1, None, 0.8, "clear map, four cards"),
    ("11_1", 3, None, 0.6, "pulse card 1"),
    ("11_2", 0, None, 5.9, "cycle_trace, shortened from 8.0"),
    ("11_2", 1, None, 0.4, "(1 3 4)"),
    ("11_2", 2, None, 0.6, "pulse card 2"),
    ("11_2", 3, None, 4.15, "ring_walk, (1 3 4)=(3 4 1)"),
    ("11_2", 4, None, 1.2, "forward arrows out, hero arrows and caption in"),
    ("11_3", 0, None, 1.2, "clear, heading, board"),
    ("11_3", 3, None, 2.9, "trace 1 (unchanged)"),
    ("11_3", 4, None, 2.9, "trace 3 (unchanged)"),
    ("11_3", 5, None, 2.9, "trace 2 (unchanged)"),
    ("11_3", 6, None, 0.4, "result (unchanged)"),
    ("11_3", 7, None, 3.8, "reverse board, all cards at once, result"),
    ("11_4", 0, None, 3.4, "clear, board, forward run, reverse run"),
    ("11_4", 1, None, 0.4, "inverse written"),
    ("11_4", 2, None, 0.8, "clear, two six-slot rows with loop arrows"),
    ("11_4", 2, "they commute", 2.2, "both orders run, equation, contrast caption"),
    ("11_4", 3, None, 3.35, "ring walks 1 to 3 to 5 to 1"),
    ("11_4", 4, None, 0.8, "closing arrow out, red arrow in"),
    ("11_4", 4, "because that would", 0.4, "red ring, warning"),
    ("11_4", 5, None, 1.4, "red out, loop closes, (1 3 5)(2 6), pulse 4"),
    ("11_4", 6, None, 0.5, "restated cycles"),
    ("11_5", 0, None, 1.2, "clear, factor, board"),
    ("11_5", 3, None, 5.2, "three traces at pace 0.6, shortened from 8.7"),
    ("11_5", 4, None, 0.4, "count caption"),
    ("11_6", 0, None, 0.6, "second description"),
    ("11_6", 1, None, 0.4, "question"),
    ("11_7", 0, None, 0.6, "question becomes the conditional"),
    ("11_7", 1, None, 0.8, "clear, product"),
    ("11_7", 2, None, 0.4, "labels and numbers"),
    ("11_7", 2, "multiply all the differences", 1.2, "six chips"),
    ("11_7", 2, "for pairs with", 0.4, "tally"),
    ("11_11", 0, None, 0.9, "numbers on labels 1 and 3 swap"),
    ("11_11", 1, None, 1.4, "chip reverses in place"),
    ("11_11", 2, None, 1.1, "blue pair trades"),
    ("11_11", 2, "sometimes both", 1.9, "gold pair trades and reverses"),
    ("11_11", 2, "so their product", 0.4, "two product notes"),
    ("11_11", 3, None, 0.4, "untouched chip dims"),
    ("11_11", 4, None, 0.8, "tally flips, sign caption"),
    ("11_8", 0, None, 4.25, "clear, two rows, three swaps with lamp"),
    ("11_8", 1, None, 1.4, "one swap, frames, nonzero"),
    ("11_8", 2, None, 0.4, "identity, both odd, impossible"),
    ("11_8", 3, None, 0.8, "the rule"),
    ("11_9", 0, None, 0.8, "clear, square"),
    ("11_9", 0, "but not every", 1.9, "trade, bow-tie, red edges, captions"),
    ("11_9", 1, None, 1.8, "clear, cards home, outline back, 24 grid"),
    ("11_9", 1, "and our eight", 0.8, "eight cells gold, count, rack"),
    ("11_9", 1, "as a subgroup", 0.4, "subgroup caption"),
    ("11_9", 2, None, 1.7, "clear, quarter-turn, r and its swaps"),
    ("11_9", 2, "a half-turn", 1.3, "second turn, r squared and its swaps"),
    ("11_9", 3, None, 1.6, "clear, cards home, second square"),
    ("11_9", 3, "r s is the swap", 2.0, "ring, s, r, rs"),
    ("11_9", 3, "and s r is the swap", 2.0, "ring, r, s, sr"),
    ("11_9", 4, None, 0.4, "tile poses, rs not sr"),
    ("11_10", 0, None, 0.8, "clear, board"),
    ("11_10", 0, "the positions of cards", 0.4, "uses caption"),
    ("11_10", 1, None, 2.6, "caption out, all cards out and back"),
    ("11_10", 2, None, 0.4, "limits caption"),
    ("11_10", 3, None, 0.3, "next-chapter caption"),
]
# Extra seconds after the line ends, from k.hold + stage.wait.
TAILS = {"11_2": 1.0, "11_3": 1.0, "11_6": 3.0}
LATE_LIMIT = 1.0


def sentences(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            out.append(text[start : i + 1].strip())
            start = i + 1
    return out


def main():
    rows = {}
    for line in (ROOT / "script.tsv").read_text(encoding="utf-8").splitlines():
        cue, _, text = line.partition("\t")
        if cue.startswith("11_"):
            rows[cue] = text
    for rate in (2.2, 2.6, 3.0):
        print(f"\nassumed rate {rate} words/s")
        total = 0.0
        for cue, text in rows.items():
            parts = sentences(text)
            starts, t = [], 0.0
            for part in parts:
                starts.append(t)
                t += len(part.split()) / rate
            spoken = t
            clock = 0.0
            lates = []
            for bcue, j, phrase, seconds, what in BEATS:
                if bcue != cue:
                    continue
                length = len(parts[j].split()) / rate
                at = starts[j] + (length * parts[j].index(phrase) / len(parts[j]) if phrase else 0.0)
                if clock - at > LATE_LIMIT:
                    lates.append(f"s{j} {what} +{clock - at:.1f}s")
                clock = max(clock, at) + seconds
            overrun = max(0.0, clock - spoken)
            tail = TAILS.get(cue, 0.0)
            total += max(spoken, clock) + tail
            print(f"  {cue:6} voice {spoken:5.1f}  picture ends {clock:5.1f}  silence after {overrun + tail:4.1f}", "; ".join(lates))
        print(f"  chapter about {total / 60:.1f} min")


if __name__ == "__main__":
    main()
