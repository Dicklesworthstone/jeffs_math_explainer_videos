"""Print the recognised words around the moments C and D depend on. Reads only; prints only.

python3 -I -B word_times.py   (from the repo root)

Each `segments` row in these files is one recognised word. Times are recognition on Oliver take01, large and small
models. They are not hearing, and they are not a statement about the take root finally selects. A sentence-final word's
end runs up to the next word, so speech_end.py measures where its sound stops.
"""
import json
from pathlib import Path

ASR = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027")
WANT = {
    "13_3": ["R", "squared.", "Three", "give", "cubed.", "Four", "return", "starting", "position.", "That"],
    "13_7": ["Whenever", "if", "we", "reverse", "the", "turn.", "Pairs"],
    "13_8": ["each", "an", "undo", "move.", "A", "quarter", "three", "turns.", "half"],
}


def main():
    for cue, words in WANT.items():
        tag = cue.replace("_", "-")
        for model in ("large", "small"):
            data = json.loads((ASR / f"expanded-algebra-{tag}-asr01/take01-{model}.json").read_text(encoding="utf-8"))
            rows = [f"{s['text'].strip()} {s['start_sec']:.2f}-{s['end_sec']:.2f}" for s in data["result"]["segments"]
                    if s["text"].strip() in words]
            print(f"{cue} {model}: " + "; ".join(rows))


if __name__ == "__main__":
    main()
